import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 20 }, // Ramp up to 20 users
    { duration: '1m', target: 20 },  // Stay at 20 users
    { duration: '10s', target: 0 },  // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p95<2000'], // P95 must be < 2s
    http_req_failed: ['rate<0.01'],  // Error rate < 1%
  },
};

export default function () {
  const url = 'http://localhost:8080/v1/chat/completions';
  const payload = JSON.stringify({
    model: "deepseek/deepseek-chat",
    messages: [
      { role: "user", content: "Hello, world!" }
    ],
    max_tokens: 10
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer test-key',
    },
  };

  const res = http.post(url, payload, params);

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response has choices': (r) => r.json('choices') !== undefined,
  });

  sleep(1);
}
