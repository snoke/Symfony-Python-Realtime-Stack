<?php

namespace App\EventListener;

use App\Service\MessageInbox;
use Psr\Log\LoggerInterface;
use Snoke\WsBundle\Event\WebsocketConnectionClosedEvent;
use Snoke\WsBundle\Event\WebsocketConnectionEstablishedEvent;
use Snoke\WsBundle\Event\WebsocketMessageReceivedEvent;
use Snoke\WsBundle\Event\WebsocketEvent;
use Symfony\Component\EventDispatcher\Attribute\AsEventListener;

#[AsEventListener(event: WebsocketConnectionEstablishedEvent::class)]
#[AsEventListener(event: WebsocketConnectionClosedEvent::class)]
#[AsEventListener(event: WebsocketMessageReceivedEvent::class)]
class WsConnectionListener
{
    public function __construct(
        private MessageInbox $inbox,
        private LoggerInterface $logger
    ) {}

    public function __invoke(WebsocketEvent $event): void
    {
        if ($event instanceof WebsocketConnectionEstablishedEvent) {
            $this->logger->info('ws.connected', [
                'connection_id' => $event->getConnectionId(),
                'user_id' => $event->getUserId(),
                'subjects' => $event->getSubjects(),
                'connected_at' => $event->getConnectedAt(),
            ]);
            return;
        }
        if ($event instanceof WebsocketConnectionClosedEvent) {
            $this->logger->info('ws.disconnected', [
                'connection_id' => $event->getConnectionId(),
                'user_id' => $event->getUserId(),
                'subjects' => $event->getSubjects(),
                'connected_at' => $event->getConnectedAt(),
            ]);
            return;
        }
        if ($event instanceof WebsocketMessageReceivedEvent) {
            $payload = [
                'connection_id' => $event->getConnectionId(),
                'user_id' => $event->getUserId(),
                'subjects' => $event->getSubjects(),
                'connected_at' => $event->getConnectedAt(),
                'message' => $event->getMessage(),
                'raw' => $event->getRaw(),
                'received_at' => time(),
            ];
            $this->inbox->setLastMessage($payload);
            $this->logger->info('ws.message_received', $payload);
        }
    }
}
