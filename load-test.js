// K6 Load Testing Script for FHE LLM Proxy
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const responseTime = new Trend('response_time');
const requestCount = new Counter('requests');

// Test configuration
export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp-up
    { duration: '2m', target: 50 },    // Stay at 50 users
    { duration: '1m', target: 100 },   // Ramp to 100 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '30s', target: 0 },    // Ramp-down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'], // 95% of requests under 500ms
    'http_req_failed': ['rate<0.1'],    // Error rate under 10%
    'errors': ['rate<0.1'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';

// Test data
const testPrompts = [
  'Explain quantum computing',
  'What is the meaning of life?',
  'How does machine learning work?',
  'Describe the theory of relativity',
  'What are the benefits of encryption?'
];

export default function () {
  // Health check
  let response = http.get(`${BASE_URL}/health`);
  check(response, {
    'health check status is 200': (r) => r.status === 200,
  });

  requestCount.add(1);
  responseTime.add(response.timings.duration);
  errorRate.add(response.status !== 200);

  sleep(0.1);

  // Metrics endpoint
  response = http.get(`${BASE_URL}/metrics`);
  check(response, {
    'metrics status is 200': (r) => r.status === 200,
  });

  requestCount.add(1);
  responseTime.add(response.timings.duration);
  errorRate.add(response.status !== 200);

  sleep(0.1);

  // FHE parameters
  response = http.get(`${BASE_URL}/v1/params`);
  check(response, {
    'params status is 200': (r) => r.status === 200,
    'params response contains poly_modulus_degree': (r) => 
      r.json().hasOwnProperty('poly_modulus_degree'),
  });

  requestCount.add(1);
  responseTime.add(response.timings.duration);
  errorRate.add(response.status !== 200);

  sleep(0.1);

  // Key generation
  response = http.post(`${BASE_URL}/v1/keys/generate`, 
    JSON.stringify({}),
    {
      headers: { 'Content-Type': 'application/json' },
    }
  );
  
  check(response, {
    'key generation status is 200': (r) => r.status === 200,
    'key generation returns client_id': (r) => 
      r.json().hasOwnProperty('client_id'),
  });

  requestCount.add(1);
  responseTime.add(response.timings.duration);
  errorRate.add(response.status !== 200);

  // Extract client_id for further tests
  let clientId = null;
  if (response.status === 200) {
    clientId = response.json().client_id;
  }

  sleep(0.2);

  // Text encryption (if we have a client_id)
  if (clientId) {
    const randomPrompt = testPrompts[Math.floor(Math.random() * testPrompts.length)];
    
    response = http.post(`${BASE_URL}/v1/encrypt`,
      JSON.stringify({
        client_id: clientId,
        plaintext: randomPrompt
      }),
      {
        headers: { 'Content-Type': 'application/json' },
      }
    );

    check(response, {
      'encryption status is 200': (r) => r.status === 200,
      'encryption returns ciphertext_id': (r) => 
        r.json().hasOwnProperty('ciphertext_id'),
    });

    requestCount.add(1);
    responseTime.add(response.timings.duration);
    errorRate.add(response.status !== 200);
  }

  sleep(0.5);
}

// Setup function
export function setup() {
  console.log('Starting FHE LLM Proxy load test...');
  console.log(`Target URL: ${BASE_URL}`);
  
  // Verify service is available
  const response = http.get(`${BASE_URL}/health`);
  if (response.status !== 200) {
    throw new Error(`Service not available. Status: ${response.status}`);
  }
  
  console.log('Service is available. Starting test...');
}

// Teardown function
export function teardown(data) {
  console.log('Load test completed.');
}