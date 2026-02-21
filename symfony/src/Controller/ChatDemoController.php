<?php

namespace App\Controller;

use Snoke\WsBundle\Contract\PresenceProviderInterface;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;

class ChatDemoController extends AbstractController
{
    public function chatPage(Request $request, PresenceProviderInterface $presence): Response
    {
        $userId = (string) $request->query->get('user_id', '');
        if ($userId === '') {
            $userId = (string) random_int(1000, 9999);
        }

        [$token, $tokenError] = $this->buildToken($userId);

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

    /**
     * @return array{0: string, 1: string}
     */
    private function buildToken(string $userId): array
    {
        $privateKeyFile = $_ENV['DEMO_JWT_PRIVATE_KEY_FILE'] ?? '';
        $secret = $_ENV['DEMO_JWT_SECRET'] ?? '';
        $alg = $_ENV['DEMO_JWT_ALG'] ?? 'RS256';

        $header = ['typ' => 'JWT', 'alg' => $alg];
        $payload = [
            'user_id' => $userId,
            'iat' => time(),
            'exp' => time() + 3600,
        ];

        $signingInput = $this->base64UrlEncode(json_encode($header)).'.'.$this->base64UrlEncode(json_encode($payload));

        $signature = '';
        if (str_starts_with($alg, 'RS')) {
            if ($privateKeyFile === '' || !is_file($privateKeyFile)) {
                return ['', 'missing DEMO_JWT_PRIVATE_KEY_FILE'];
            }
            $key = file_get_contents($privateKeyFile);
            if ($key === false) {
                return ['', 'failed to read private key'];
            }
            $ok = openssl_sign($signingInput, $signature, $key, OPENSSL_ALGO_SHA256);
            if (!$ok) {
                return ['', 'failed to sign token'];
            }
        } else {
            if ($secret === '') {
                return ['', 'missing DEMO_JWT_SECRET'];
            }
            $signature = hash_hmac('sha256', $signingInput, $secret, true);
        }

        $jwt = $signingInput.'.'.$this->base64UrlEncode($signature);

        return [$jwt, ''];
    }

    private function base64UrlEncode(string $data): string
    {
        return rtrim(strtr(base64_encode($data), '+/', '-_'), '=');
    }
}
