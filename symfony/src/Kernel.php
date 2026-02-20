<?php

namespace App;

use Symfony\Bundle\FrameworkBundle\Kernel\MicroKernelTrait;
use Symfony\Component\Routing\Loader\Configurator\RoutingConfigurator;
use Symfony\Component\HttpKernel\Kernel as BaseKernel;

class Kernel extends BaseKernel
{
    use MicroKernelTrait;

    protected function configureRoutes(RoutingConfigurator $routes): void
    {
        $routes->import('../config/routes/*.yaml');
        $routes->import('../config/routes/{env}/*.yaml');
        $routes->import('../src/Controller/', 'attribute');
        $routes->import('@SnokeWsBundle/Controller/', 'attribute');
    }
}
