<?php

namespace App\EventListener;

use Psr\Log\LoggerInterface;
use Snoke\WsBundle\Event\WebsocketConnectionClosedEvent;
use Snoke\WsBundle\Event\WebsocketConnectionEstablishedEvent;
use Symfony\Component\EventDispatcher\Attribute\AsEventListener;

#[AsEventListener(event: WebsocketConnectionEstablishedEvent::class)]
#[AsEventListener(event: WebsocketConnectionClosedEvent::class)]
class WsConnectionListener
{
    public function __construct(
        private LoggerInterface $logger
    ) {}

    public function __invoke(object $event): void
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
        }
    }
}
