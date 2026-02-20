<?php

namespace Snoke\WsBundle\Event;

use Symfony\Contracts\EventDispatcher\Event;

abstract class WebsocketEvent extends Event
{
    public function __construct(
        private string $connectionId,
        private string $userId,
        private array $subjects,
        private int $connectedAt
    ) {}

    public function getConnectionId(): string
    {
        return $this->connectionId;
    }

    public function getUserId(): string
    {
        return $this->userId;
    }

    public function getSubjects(): array
    {
        return $this->subjects;
    }

    public function getConnectedAt(): int
    {
        return $this->connectedAt;
    }
}
