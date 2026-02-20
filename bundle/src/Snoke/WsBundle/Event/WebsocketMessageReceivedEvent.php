<?php

namespace Snoke\WsBundle\Event;

class WebsocketMessageReceivedEvent extends WebsocketEvent
{
    public function __construct(
        string $connectionId,
        string $userId,
        array $subjects,
        int $connectedAt,
        private mixed $message,
        private string $raw
    ) {
        parent::__construct($connectionId, $userId, $subjects, $connectedAt);
    }

    public function getMessage(): mixed
    {
        return $this->message;
    }

    public function getRaw(): string
    {
        return $this->raw;
    }
}
