<?php

namespace App\Controller;

use Snoke\WsBundle\Contract\PresenceProviderInterface;
use Snoke\WsBundle\Service\DemoTokenService;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;

class ChatDemoController extends AbstractController
{
    public function __construct(private DemoTokenService $tokenService)
    {
    }

    public function chatPage(Request $request, PresenceProviderInterface $presence): Response
    {
        $userId = (string) $request->query->get('user_id', '');
        if ($userId === '') {
            $userId = (string) random_int(1000, 9999);
        }

        [$token, $tokenError] = $this->tokenService->issue($userId);

        $initialUsers = [];
        try {
            $connections = $presence->listConnections();
            if (isset($connections['connections']) && is_array($connections['connections'])) {
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
                $initialUsers = array_keys($seen);
            }
        } catch (\Throwable) {
            $initialUsers = [];
        }

        return $this->render('demo/chat.html.twig', [
            'user_id' => $userId,
            'token' => $token,
            'token_error' => $tokenError,
            'initial_users' => $initialUsers,
        ]);
    }
}
