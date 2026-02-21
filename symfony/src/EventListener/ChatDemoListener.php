<?php

namespace App\EventListener;

use Snoke\WsBundle\Contract\PresenceProviderInterface;
use Snoke\WsBundle\Event\WebsocketConnectionClosedEvent;
use Snoke\WsBundle\Event\WebsocketConnectionEstablishedEvent;
use Snoke\WsBundle\Event\WebsocketMessageReceivedEvent;
use Snoke\WsBundle\Service\WebsocketPublisher;
use Symfony\Component\EventDispatcher\Attribute\AsEventListener;

class ChatDemoListener
{
    public function __construct(
        private WebsocketPublisher $publisher,
        private PresenceProviderInterface $presence
    ) {}

    #[AsEventListener(event: WebsocketMessageReceivedEvent::class)]
    public function onMessage(WebsocketMessageReceivedEvent $event): void
    {
        $message = $event->getMessage();
        if (!is_array($message)) {
            return;
        }
        if (($message['type'] ?? '') !== 'chat') {
            return;
        }
        $text = trim((string) ($message['text'] ?? ''));
        if ($text === '') {
            return;
        }

        $payload = [
            'type' => 'chat',
            'user' => 'user:'.$event->getUserId(),
            'text' => $text,
            'ts' => time(),
        ];

        $targets = $this->resolveTargets($event->getUserId());
        if ($targets === []) {
            return;
        }
        $this->publisher->send($targets, $payload);
    }

    #[AsEventListener(event: WebsocketConnectionEstablishedEvent::class)]
    public function onConnected(WebsocketConnectionEstablishedEvent $event): void
    {
        $this->broadcastPresence();
    }

    #[AsEventListener(event: WebsocketConnectionClosedEvent::class)]
    public function onClosed(WebsocketConnectionClosedEvent $event): void
    {
        $this->broadcastPresence();
    }

    private function broadcastPresence(): void
    {
        $users = $this->collectUsers();
        if ($users === []) {
            return;
        }
        $payload = [
            'type' => 'presence',
            'users' => $users,
            'ts' => time(),
        ];
        $this->publisher->send($users, $payload);
    }

    /**
     * @return array<int, string>
     */
    private function resolveTargets(string $fallbackUserId): array
    {
        $users = $this->collectUsers();
        $users[] = 'user:'.$fallbackUserId;
        return array_values(array_unique($users));
    }

    /**
     * @return array<int, string>
     */
    private function collectUsers(): array
    {
        try {
            $connections = $this->presence->listConnections();
        } catch (\Throwable) {
            return [];
        }
        if (!isset($connections['connections']) || !is_array($connections['connections'])) {
            return [];
        }
        $seen = [];
        foreach ($connections['connections'] as $conn) {
            if (!is_array($conn)) {
                continue;
            }
            $uid = $conn['user_id'] ?? null;
            if ($uid) {
                $seen['user:'.$uid] = true;
            }
        }
        return array_keys($seen);
    }
}
