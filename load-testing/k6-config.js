// K6 load testing configuration for FHE LLM Proxy
// Tests encryption, homomorphic operations, and overall system performance

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics for FHE operations
const encryptionLatency = new Trend('encryption_latency_ms');
const decryptionLatency = new Trend('decryption_latency_ms');
const privacyBudgetUsage = new Counter('privacy_budget_consumed');
const fheErrors = new Rate('fhe_error_rate');

// Test configuration options
export const options = {
  scenarios: {
    // Baseline load test - normal traffic patterns
    baseline_load: {
      executor: 'constant-vus',
      vus: 10,                    // 10 virtual users
      duration: '5m',             // Run for 5 minutes
      tags: { test_type: 'baseline' },
    },
    
    // Spike test - sudden traffic increases
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 5 },    // Ramp up
        { duration: '30s', target: 50 },  // Spike
        { duration: '1m', target: 5 },    // Ramp down
      ],
      tags: { test_type: 'spike' },
    },
    
    // Stress test - sustained high load
    stress_test: {
      executor: 'ramping-vus', 
      startVUs: 0,
      stages: [
        { duration: '2m', target: 20 },   // Ramp up
        { duration: '5m', target: 20 },   // Sustain
        { duration: '2m', target: 0 },    // Ramp down
      ],
      tags: { test_type: 'stress' },
    },
    
    // Privacy budget test - test budget exhaustion
    privacy_budget_test: {
      executor: 'shared-iterations',
      iterations: 1000,              // Fixed number of requests
      vus: 5,
      maxDuration: '10m',
      tags: { test_type: 'privacy_budget' },
    }
  },
  
  // Performance thresholds
  thresholds: {
    http_req_duration: ['p(95)<5000'],        // 95% under 5s
    http_req_failed: ['rate<0.05'],           // Error rate under 5%
    encryption_latency_ms: ['p(90)<2000'],    // 90% encrypt under 2s
    decryption_latency_ms: ['p(90)<1000'],    // 90% decrypt under 1s
    fhe_error_rate: ['rate<0.01'],           // FHE error rate under 1%
  }
};

// Test data - sample prompts for FHE encryption
const testPrompts = [
  "Explain quantum computing in simple terms",
  "What are the benefits of homomorphic encryption?", 
  "How does machine learning work?",
  "Describe the process of photosynthesis",
  "What is the future of artificial intelligence?",
  "Compare renewable energy sources",
  "Explain blockchain technology",
  "What are the causes of climate change?"
];

// Base URL for the FHE proxy service
const BASE_URL = __ENV.FHE_PROXY_URL || 'http://localhost:8080';

// Authentication token (if required)
const AUTH_TOKEN = __ENV.FHE_AUTH_TOKEN || '';

// Test setup - run once per VU
export function setup() {
  console.log('Starting FHE LLM Proxy load test...');
  console.log(`Target URL: ${BASE_URL}`);
  
  // Health check before starting tests
  const healthCheck = http.get(`${BASE_URL}/health`);
  check(healthCheck, {
    'health check successful': (r) => r.status === 200,
  });
  
  return { 
    baseUrl: BASE_URL,
    prompts: testPrompts 
  };
}

// Main test function
export default function(data) {
  const headers = {
    'Content-Type': 'application/json',
    'User-Agent': 'K6-LoadTest/1.0',
  };
  
  if (AUTH_TOKEN) {
    headers['Authorization'] = `Bearer ${AUTH_TOKEN}`;
  }
  
  // Select random prompt for variety
  const prompt = data.prompts[Math.floor(Math.random() * data.prompts.length)];
  
  // Test payload
  const payload = JSON.stringify({
    messages: [
      {
        role: "user",
        content: prompt
      }
    ],
    model: "gpt-3.5-turbo",
    temperature: 0.7,
    max_tokens: 150,
    // FHE-specific parameters
    fhe_config: {
      privacy_budget: 0.1,
      key_rotation: false,
      encryption_scheme: "CKKS"
    }
  });
  
  // Measure encryption phase
  const encryptStart = Date.now();
  
  // Make request to FHE proxy
  const response = http.post(`${data.baseUrl}/v1/chat/completions`, payload, {
    headers: headers,
    timeout: '30s',
    tags: { 
      operation: 'fhe_chat_completion',
      scenario: __ENV.scenario || 'unknown'
    }
  });
  
  const encryptEnd = Date.now();
  const totalLatency = encryptEnd - encryptStart;
  
  // Record encryption latency (includes homomorphic operations)
  encryptionLatency.add(totalLatency);
  
  // Validate response
  const responseChecks = check(response, {
    'status is 200': (r) => r.status === 200,
    'response has body': (r) => r.body && r.body.length > 0,
    'response is JSON': (r) => {
      try {
        JSON.parse(r.body);
        return true;
      } catch {
        return false;
      }
    },
    'encryption completed': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body.choices && body.choices.length > 0;
      } catch {
        return false;
      }
    }
  });
  
  // Track FHE-specific errors
  if (response.status !== 200) {
    fheErrors.add(1);
  }
  
  // Parse response for FHE metadata
  if (response.status === 200) {
    try {
      const responseBody = JSON.parse(response.body);
      
      // Track privacy budget consumption if available
      if (responseBody.fhe_metadata && responseBody.fhe_metadata.privacy_budget_consumed) {
        privacyBudgetUsage.add(responseBody.fhe_metadata.privacy_budget_consumed);
      }
      
      // Track decryption time if available
      if (responseBody.fhe_metadata && responseBody.fhe_metadata.decryption_time_ms) {
        decryptionLatency.add(responseBody.fhe_metadata.decryption_time_ms);
      }
      
    } catch (e) {
      console.error('Failed to parse response JSON:', e);
      fheErrors.add(1);
    }
  }
  
  // Realistic user behavior - think time between requests
  sleep(Math.random() * 2 + 1); // 1-3 seconds
}

// Teardown function - run once after all iterations
export function teardown(data) {
  console.log('Load test completed');
  
  // Final health check
  const finalHealthCheck = http.get(`${data.baseUrl}/health`);
  check(finalHealthCheck, {
    'final health check successful': (r) => r.status === 200,
  });
  
  // Get final metrics if available
  const metricsResponse = http.get(`${data.baseUrl}/metrics`);
  if (metricsResponse.status === 200) {
    console.log('Final system metrics retrieved');
  }
}

// Helper function for GPU memory stress test
export function gpuStressTest() {
  const largePrompt = "A".repeat(10000); // Large prompt to stress GPU memory
  
  const payload = JSON.stringify({
    messages: [{ role: "user", content: largePrompt }],
    model: "gpt-4",
    max_tokens: 1000,
    fhe_config: {
      privacy_budget: 0.5,
      poly_modulus_degree: 32768, // Large polynomial for GPU stress
      encryption_scheme: "CKKS"
    }
  });
  
  const response = http.post(`${BASE_URL}/v1/chat/completions`, payload, {
    headers: { 'Content-Type': 'application/json' },
    timeout: '60s',
    tags: { operation: 'gpu_stress_test' }
  });
  
  check(response, {
    'GPU stress test completed': (r) => r.status === 200,
    'no GPU memory errors': (r) => !r.body.includes('CUDA_ERROR_OUT_OF_MEMORY'),
  });
}

// Privacy budget exhaustion test
export function privacyBudgetExhaustionTest() {
  // Make multiple requests to exhaust privacy budget
  for (let i = 0; i < 20; i++) {
    const payload = JSON.stringify({
      messages: [{ role: "user", content: "Test privacy budget" }],
      fhe_config: { privacy_budget: 0.8 } // High privacy budget consumption
    });
    
    const response = http.post(`${BASE_URL}/v1/chat/completions`, payload, {
      headers: { 'Content-Type': 'application/json' },
      tags: { operation: 'privacy_budget_test' }
    });
    
    // Should eventually get privacy budget exhausted error
    if (response.status === 429) {
      check(response, {
        'privacy budget exhausted correctly': (r) => 
          r.body.includes('privacy budget') || r.body.includes('rate limit')
      });
      break;
    }
  }
}