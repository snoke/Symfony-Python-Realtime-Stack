<?php

namespace Snoke\WsBundle\DependencyInjection;

use Symfony\Component\DependencyInjection\ContainerBuilder;
use Symfony\Component\DependencyInjection\Extension\Extension;
use Symfony\Component\DependencyInjection\Reference;

class SnokeWsExtension extends Extension
{
    public function load(array $configs, ContainerBuilder $container): void
    {
        $configuration = new Configuration();
        $config = $this->processConfiguration($configuration, $configs);

        $resolveEnvValue = static function (?string $value, string $envName): ?string {
            if ($value === null) {
                return null;
            }
            if (is_string($value) && str_starts_with($value, 'env_')) {
                $env = $_ENV[$envName] ?? getenv($envName);
                if ($env === false || $env === null || $env === '') {
                    return null;
                }
                return (string) $env;
            }
            return $value;
        };

        $rawConfig = $config;

        $resolvedMode = $resolveEnvValue($config['mode'] ?? null, 'WS_MODE') ?? 'terminator';

        $resolvedTransportType = $resolveEnvValue($config['transport']['type'] ?? null, 'WS_TRANSPORT_TYPE');
        if (!$resolvedTransportType) {
            $resolvedTransportType = $resolvedMode === 'core' ? 'redis_stream' : 'http';
        }
        if (empty($rawConfig['transport']['type'])) {
            $rawConfig['transport']['type'] = $resolvedTransportType;
        }

        $resolvedPresenceType = $resolveEnvValue($config['presence']['type'] ?? null, 'WS_PRESENCE_TYPE');
        if (!$resolvedPresenceType) {
            $resolvedPresenceType = $resolvedMode === 'core' ? 'redis' : 'http';
        }
        if (empty($rawConfig['presence']['type'])) {
            $rawConfig['presence']['type'] = $resolvedPresenceType;
        }

        $resolvedEventsType = $resolveEnvValue($config['events']['type'] ?? null, 'WS_EVENTS_TYPE');
        if (!$resolvedEventsType) {
            $resolvedEventsType = $resolvedMode === 'core' ? 'redis_stream' : 'webhook';
        }
        if (empty($rawConfig['events']['type'])) {
            $rawConfig['events']['type'] = $resolvedEventsType;
        }

        $mode = $resolvedMode;
        $transportType = $resolvedTransportType;
        $presenceType = $resolvedPresenceType;
        $eventsType = $resolvedEventsType;

        if (empty($rawConfig['mode'])) {
            $rawConfig['mode'] = $resolvedMode;
        }

        $container->setParameter('snoke_ws.mode', $rawConfig['mode']);
        $container->setParameter('snoke_ws.transport', $rawConfig['transport']);
        $container->setParameter('snoke_ws.presence', $rawConfig['presence']);
        $container->setParameter('snoke_ws.events', $rawConfig['events']);
        $container->setParameter('snoke_ws.tracing', $rawConfig['tracing']);
        $container->setParameter('snoke_ws.subjects', $rawConfig['subjects']);

        $container->register('snoke_ws.http_publisher', 'Snoke\\WsBundle\\Service\\HttpPublisher')
            ->addArgument(new Reference('http_client'))
            ->addArgument('%snoke_ws.transport%')
            ->addArgument(new Reference('snoke_ws.tracing'));
        $container->register('snoke_ws.redis_stream_publisher', 'Snoke\\WsBundle\\Service\\RedisStreamPublisher')
            ->addArgument('%snoke_ws.transport%')
            ->addArgument(new Reference('snoke_ws.tracing'));
        $container->register('snoke_ws.rabbitmq_publisher', 'Snoke\\WsBundle\\Service\\RabbitMqPublisher')
            ->addArgument('%snoke_ws.transport%')
            ->addArgument(new Reference('snoke_ws.tracing'));

        $container->register('snoke_ws.dynamic_publisher', 'Snoke\\WsBundle\\Service\\DynamicPublisher')
            ->addArgument(new Reference('snoke_ws.http_publisher'))
            ->addArgument(new Reference('snoke_ws.redis_stream_publisher'))
            ->addArgument(new Reference('snoke_ws.rabbitmq_publisher'))
            ->addArgument('%snoke_ws.transport%');

        if ($presenceType === 'http') {
            $container->register('snoke_ws.http_presence', 'Snoke\\WsBundle\\Service\\HttpPresenceProvider')
                ->addArgument(new Reference('http_client'))
                ->addArgument('%snoke_ws.presence%');
            $presenceService = 'snoke_ws.http_presence';
        } else {
            $container->register('snoke_ws.redis_presence', 'Snoke\\WsBundle\\Service\\RedisPresenceProvider')
                ->addArgument('%snoke_ws.presence%');
            $presenceService = 'snoke_ws.redis_presence';
        }

        $writerType = $config['presence']['writer']['type'] ?? 'none';
        if ($writerType === 'redis') {
            $container->register('snoke_ws.presence_writer', 'Snoke\\WsBundle\\Service\\RedisPresenceWriter')
                ->addArgument('%snoke_ws.presence%');
        } else {
            $container->register('snoke_ws.presence_writer', 'Snoke\\WsBundle\\Service\\NullPresenceWriter');
        }
        $container->setAlias('Snoke\\WsBundle\\Contract\\PresenceWriterInterface', 'snoke_ws.presence_writer');
        $container->register('snoke_ws.presence_writer_listener', 'Snoke\\WsBundle\\EventListener\\PresenceWriterListener')
            ->addArgument(new Reference('snoke_ws.presence_writer'))
            ->addArgument('%snoke_ws.presence%')
            ->setAutowired(true)
            ->setAutoconfigured(true);

        $container->register('snoke_ws.subject_key_resolver', 'Snoke\\WsBundle\\Service\\SimpleSubjectKeyResolver')
            ->addArgument('%snoke_ws.subjects%');

        $container->register('snoke_ws.publisher', 'Snoke\\WsBundle\\Service\\WebsocketPublisher')
            ->addArgument(new Reference('snoke_ws.dynamic_publisher'))
            ->addArgument(new Reference('snoke_ws.subject_key_resolver'));
        $container->setAlias('Snoke\\WsBundle\\Service\\WebsocketPublisher', 'snoke_ws.publisher');

        $container->setAlias('Snoke\\WsBundle\\Contract\\PresenceProviderInterface', $presenceService);

        $container->register('snoke_ws.tracing', 'Snoke\\WsBundle\\Service\\TracingService')
            ->addArgument('%snoke_ws.tracing%');
        $container->setAlias('Snoke\\WsBundle\\Service\\TracingService', 'snoke_ws.tracing');

        $container->register('Snoke\\WsBundle\\Controller\\WebhookController', 'Snoke\\WsBundle\\Controller\\WebhookController')
            ->addArgument(new Reference('event_dispatcher'))
            ->addArgument('%snoke_ws.events%')
            ->addArgument(new Reference('snoke_ws.tracing'))
            ->addTag('controller.service_arguments');
        $container->setAlias('snoke_ws.webhook_controller', 'Snoke\\WsBundle\\Controller\\WebhookController');
    }
}
