#!/usr/bin/env php
<?php

use App\Kernel;
use App\Service\WsInboxConsumer;

require dirname(__DIR__).'/vendor/autoload.php';

$kernel = new Kernel('dev', true);
$kernel->boot();

$container = $kernel->getContainer();
/** @var WsInboxConsumer $consumer */
$consumer = $container->get(WsInboxConsumer::class);
$consumer->run();
