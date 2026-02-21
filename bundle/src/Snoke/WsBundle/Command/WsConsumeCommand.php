<?php

namespace Snoke\WsBundle\Command;

use Snoke\WsBundle\Service\InboxConsumer;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

#[AsCommand(name: 'ws:consume', description: 'Consume ws.inbox streams and dispatch events')]
class WsConsumeCommand extends Command
{
    public function __construct(private InboxConsumer $consumer)
    {
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $this->consumer->run();
        return Command::SUCCESS;
    }
}
