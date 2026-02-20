<?php

namespace App\Service;

use Predis\Client;

class MessageInbox
{
    private ?array $lastMessage = null;
    private ?Client $redis = null;
    private string $redisKey = 'demo:ws:last_message';

    public function __construct()
    {
        $dsn = $_ENV['DEMO_INBOX_REDIS_DSN'] ?? '';
        if ($dsn !== '') {
            $this->redis = new Client($dsn);
        }
    }

    public function setLastMessage(array $payload): void
    {
        $this->lastMessage = $payload;
        if ($this->redis) {
            $this->redis->set($this->redisKey, json_encode($payload));
        }
    }

    public function getLastMessage(): ?array
    {
        if ($this->redis) {
            $value = $this->redis->get($this->redisKey);
            if ($value) {
                $decoded = json_decode($value, true);
                if (is_array($decoded)) {
                    return $decoded;
                }
            }
        }
        return $this->lastMessage;
    }
}
