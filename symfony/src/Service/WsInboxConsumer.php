<?php

namespace App\Service;

use Predis\Client;
use Psr\Log\LoggerInterface;
use Snoke\WsBundle\Event\WebsocketConnectionClosedEvent;
use Snoke\WsBundle\Event\WebsocketConnectionEstablishedEvent;
use Snoke\WsBundle\Event\WebsocketMessageReceivedEvent;
use Snoke\WsBundle\Service\TracingService;
use OpenTelemetry\API\Trace\SpanKind;
use Symfony\Contracts\EventDispatcher\EventDispatcherInterface;

class WsInboxConsumer
{
    private int $minLevel;

    public function __construct(
        private EventDispatcherInterface $dispatcher,
        private LoggerInterface $logger,
        private TracingService $tracing
    ) {
        $this->minLevel = $this->parseLevel($_ENV['WS_CONSUMER_LOG_LEVEL'] ?? 'info');
    }

    public function run(): void
    {
        $dsn = $_ENV['DEMO_INBOX_REDIS_DSN']
            ?? $_ENV['WS_REDIS_DSN']
            ?? $_ENV['REDIS_DSN']
            ?? '';
        if ($dsn === '') {
            $this->logger->error('ws.inbox.missing_redis_dsn');
            return;
        }

        $inboxStream = $_ENV['DEMO_INBOX_REDIS_STREAM']
            ?? $_ENV['WS_REDIS_INBOX_STREAM']
            ?? $_ENV['REDIS_INBOX_STREAM']
            ?? 'ws.inbox';

        $client = new Client($dsn);

        $streamNames = $this->resolveStreams($inboxStream);
        if ($streamNames === []) {
            $this->logger->error('ws.inbox.missing_streams');
            return;
        }

        $lastIds = $this->resolveStartIds($client, $streamNames);
        $streamIndex = array_flip($streamNames);

        $this->logInfo('ws.inbox.consumer_started', [
            'streams' => $streamNames,
            'last_ids' => $lastIds,
        ]);

        while (true) {
            try {
                $response = null;
                set_error_handler(static function (): bool {
                    return true;
                });
                try {
                    $response = $client->xread(10, 5000, $streamNames, ...$lastIds);
                } finally {
                    restore_error_handler();
                }
                if (!$response || !is_array($response)) {
                    continue;
                }
                foreach ($response as $stream => $entries) {
                    if (!isset($streamIndex[$stream]) || !is_array($entries)) {
                        continue;
                    }
                    foreach ($this->normalizeEntries($entries) as [$entryId, $fields]) {
                        $entryId = (string) $entryId;
                        $this->handleEvent($fields);
                        $lastIds[$streamIndex[$stream]] = $entryId;
                    }
                }
            } catch (\Throwable $e) {
                $this->logger->error('ws.inbox.consume_error: '.$e->getMessage());
                sleep(1);
            }
        }
    }

    /**
     * @return array<int, string>
     */
    private function resolveStreams(string $inboxStream): array
    {
        $streams = [];
        if ($inboxStream !== '') {
            $streams[] = $inboxStream;
        }

        $eventsType = (string) ($_ENV['WS_EVENTS_TYPE'] ?? '');
        $eventsStream = (string) ($_ENV['WS_REDIS_EVENTS_STREAM']
            ?? $_ENV['REDIS_EVENTS_STREAM']
            ?? 'ws.events');
        if ($eventsStream !== '' && $eventsStream !== $inboxStream && ($eventsType === '' || $eventsType === 'redis_stream')) {
            $streams[] = $eventsStream;
        }

        return array_values(array_unique($streams));
    }

    /**
     * @param array<int, string> $streamNames
     * @return array<int, string>
     */
    private function resolveStartIds(Client $client, array $streamNames): array
    {
        $lastIds = [];
        foreach ($streamNames as $stream) {
            $lastId = '0-0';
            try {
                $latest = $client->xrevrange($stream, '+', '-', 1);
                if (is_array($latest) && $latest !== []) {
                    $key = array_key_first($latest);
                    if (is_string($key) && $key !== '') {
                        $lastId = $key;
                    }
                }
            } catch (\Throwable) {
                $lastId = '0-0';
            }
            $lastIds[] = $lastId;
        }
        return $lastIds;
    }

    private function handleEvent(array $fields): void
    {
        $raw = $fields['data'] ?? '';
        if (!is_string($raw) || $raw === '') {
            return;
        }
        $event = json_decode($raw, true);
        if (!is_array($event)) {
            return;
        }
        $type = (string) ($event['type'] ?? '');
        $connectionId = (string) ($event['connection_id'] ?? '');
        $userId = (string) ($event['user_id'] ?? '');
        $subjects = $event['subjects'] ?? [];
        if (!is_array($subjects)) {
            $subjects = [];
        }
        $connectedAt = (int) ($event['connected_at'] ?? 0);
        $traceparentField = $this->tracing->getTraceparentField();
        $traceparent = $event[$traceparentField] ?? null;
        if (!is_string($traceparent)) {
            $traceparent = null;
        }

        $scope = $this->tracing->startSpan('ws.inbox.consume', SpanKind::KIND_CONSUMER, [
            'ws.event_type' => $type,
            'ws.connection_id' => $connectionId,
            'ws.user_id' => $userId,
        ], $traceparent);

        try {
            if ($type === 'message_received') {
                $message = $event['message'] ?? null;
                $rawMessage = (string) ($event['raw'] ?? '');
                $this->dispatcher->dispatch(new WebsocketMessageReceivedEvent(
                    $connectionId,
                    $userId,
                    $subjects,
                    $connectedAt,
                    $message,
                    $rawMessage
                ));
                return;
            }
            if ($type === 'connected') {
                $this->dispatcher->dispatch(new WebsocketConnectionEstablishedEvent(
                    $connectionId,
                    $userId,
                    $subjects,
                    $connectedAt
                ));
                return;
            }
            if ($type === 'disconnected') {
                $this->dispatcher->dispatch(new WebsocketConnectionClosedEvent(
                    $connectionId,
                    $userId,
                    $subjects,
                    $connectedAt
                ));
            }
        } finally {
            if ($scope) {
                $scope->end();
            }
        }
    }

    /**
     * @return array<int, array{0: string, 1: array}>
     */
    private function normalizeEntries(mixed $entries): array
    {
        if (!is_array($entries)) {
            return [];
        }
        $normalized = [];
        foreach ($entries as $key => $value) {
            if (is_array($value) && array_key_exists(0, $value) && array_key_exists(1, $value)) {
                $normalized[] = [$value[0], $this->normalizeFields($value[1])];
                continue;
            }
            if (is_string($key) && is_array($value)) {
                $normalized[] = [$key, $this->normalizeFields($value)];
            }
        }
        return $normalized;
    }

    private function normalizeFields(array $fields): array
    {
        if ($fields === []) {
            return $fields;
        }
        $keys = array_keys($fields);
        if ($keys !== range(0, count($fields) - 1)) {
            return $fields;
        }
        $assoc = [];
        $count = count($fields);
        for ($i = 0; $i + 1 < $count; $i += 2) {
            $assoc[(string) $fields[$i]] = $fields[$i + 1];
        }
        return $assoc;
    }

    private function logInfo(string $message, array $context = []): void
    {
        if ($this->minLevel <= 200) {
            $this->logger->info($message, $context);
        }
    }

    private function parseLevel(string $level): int
    {
        $level = strtolower($level);
        return match ($level) {
            'debug' => 100,
            'info' => 200,
            'notice' => 250,
            'warning' => 300,
            'error' => 400,
            'critical' => 500,
            'alert' => 550,
            'emergency' => 600,
            default => 200,
        };
    }
}
