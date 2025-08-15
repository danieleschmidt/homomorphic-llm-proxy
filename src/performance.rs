//! Advanced performance optimization and caching systems

use crate::error::Result;
use crate::fhe::{Ciphertext, FheEngine};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Advanced multi-level caching system
pub struct PerformanceCache {
    l1_cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    l2_cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    hot_cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
    stats: Arc<CacheStats>,
    config: CacheConfig,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub operation_type: String,
    pub input_hash: u64,
    pub params_hash: u64,
    pub client_id: Uuid,
}

#[derive(Debug)]
pub struct CacheEntry {
    pub data: CacheData,
    pub created_at: Instant,
    pub accessed_at: Instant,
    pub access_count: AtomicU64,
    pub size_bytes: usize,
    pub ttl: Duration,
    pub priority: CachePriority,
}

#[derive(Debug, Clone)]
pub enum CacheData {
    Ciphertext(Ciphertext),
    Result(String),
    Intermediate(Vec<u8>),
    Metadata(HashMap<String, String>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub l1_max_size: usize,
    pub l2_max_size: usize,
    pub hot_max_size: usize,
    pub default_ttl: Duration,
    pub hot_threshold_accesses: u64,
    pub eviction_strategy: EvictionStrategy,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum EvictionStrategy {
    LRU,
    LFU,
    TLRU, // Time-aware LRU
    Adaptive,
}

#[derive(Debug)]
pub struct CacheStats {
    pub l1_hits: AtomicU64,
    pub l1_misses: AtomicU64,
    pub l2_hits: AtomicU64,
    pub l2_misses: AtomicU64,
    pub hot_hits: AtomicU64,
    pub evictions: AtomicU64,
    pub compressions: AtomicU64,
    pub total_size_bytes: AtomicUsize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 1024 * 1024 * 100, // 100MB
            l2_max_size: 1024 * 1024 * 500, // 500MB
            hot_max_size: 1024 * 1024 * 50,  // 50MB
            default_ttl: Duration::from_secs(3600),
            hot_threshold_accesses: 10,
            eviction_strategy: EvictionStrategy::Adaptive,
            compression_enabled: true,
            encryption_enabled: false, // Would require additional setup
        }
    }
}

impl PerformanceCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(HashMap::new())),
            l2_cache: Arc::new(RwLock::new(HashMap::new())),
            hot_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(CacheStats {
                l1_hits: AtomicU64::new(0),
                l1_misses: AtomicU64::new(0),
                l2_hits: AtomicU64::new(0),
                l2_misses: AtomicU64::new(0),
                hot_hits: AtomicU64::new(0),
                evictions: AtomicU64::new(0),
                compressions: AtomicU64::new(0),
                total_size_bytes: AtomicUsize::new(0),
            }),
            config,
        }
    }

    /// Multi-level cache lookup with intelligent promotion
    pub async fn get(&self, key: &CacheKey) -> Option<CacheData> {
        // Check hot cache first (most frequently accessed)
        if let Some(entry) = self.get_from_cache(&self.hot_cache, key).await {
            self.stats.hot_hits.fetch_add(1, Ordering::Relaxed);
            return Some(entry);
        }

        // Check L1 cache
        if let Some(entry) = self.get_from_cache(&self.l1_cache, key).await {
            self.stats.l1_hits.fetch_add(1, Ordering::Relaxed);
            self.maybe_promote_to_hot(key, &entry).await;
            return Some(entry);
        }
        
        self.stats.l1_misses.fetch_add(1, Ordering::Relaxed);

        // Check L2 cache
        if let Some(entry) = self.get_from_cache(&self.l2_cache, key).await {
            self.stats.l2_hits.fetch_add(1, Ordering::Relaxed);
            self.promote_to_l1(key.clone(), entry.clone()).await;
            return Some(entry);
        }
        
        self.stats.l2_misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    async fn get_from_cache(
        &self,
        cache: &Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
        key: &CacheKey,
    ) -> Option<CacheData> {
        let mut cache_map = cache.write().await;
        
        if let Some(entry) = cache_map.get_mut(key) {
            // Check TTL
            if entry.created_at.elapsed() > entry.ttl {
                cache_map.remove(key);
                return None;
            }

            // Update access information
            entry.accessed_at = Instant::now();
            entry.access_count.fetch_add(1, Ordering::Relaxed);
            
            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Store data in appropriate cache level
    pub async fn put(&self, key: CacheKey, data: CacheData, priority: CachePriority) {
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            accessed_at: Instant::now(),
            access_count: AtomicU64::new(1),
            size_bytes: self.estimate_size(&key),
            ttl: self.config.default_ttl,
            priority: priority.clone(),
        };

        match priority {
            CachePriority::Critical => {
                self.put_in_cache(&self.hot_cache, key, entry).await;
            }
            CachePriority::High => {
                self.put_in_cache(&self.l1_cache, key, entry).await;
            }
            _ => {
                self.put_in_cache(&self.l2_cache, key, entry).await;
            }
        }
    }

    async fn put_in_cache(
        &self,
        cache: &Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
        key: CacheKey,
        entry: CacheEntry,
    ) {
        let mut cache_map = cache.write().await;
        
        // Check if eviction is needed
        if self.needs_eviction(&cache_map, entry.size_bytes).await {
            self.evict_entries(&mut cache_map).await;
        }

        cache_map.insert(key, entry);
    }

    async fn maybe_promote_to_hot(&self, key: &CacheKey, entry: &CacheData) {
        // Check if this entry should be promoted to hot cache
        if let Some(cache_entry) = self.l1_cache.read().await.get(key) {
            let access_count = cache_entry.access_count.load(Ordering::Relaxed);
            if access_count >= self.config.hot_threshold_accesses {
                let hot_entry = CacheEntry {
                    data: entry.clone(),
                    created_at: cache_entry.created_at,
                    accessed_at: Instant::now(),
                    access_count: AtomicU64::new(access_count),
                    size_bytes: cache_entry.size_bytes,
                    ttl: cache_entry.ttl,
                    priority: CachePriority::Critical,
                };
                
                self.put_in_cache(&self.hot_cache, key.clone(), hot_entry).await;
            }
        }
    }

    async fn promote_to_l1(&self, key: CacheKey, entry: CacheData) {
        if let Some(cache_entry) = self.l2_cache.read().await.get(&key) {
            let l1_entry = CacheEntry {
                data: entry,
                created_at: cache_entry.created_at,
                accessed_at: Instant::now(),
                access_count: AtomicU64::new(cache_entry.access_count.load(Ordering::Relaxed)),
                size_bytes: cache_entry.size_bytes,
                ttl: cache_entry.ttl,
                priority: CachePriority::High,
            };
            
            self.put_in_cache(&self.l1_cache, key, l1_entry).await;
        }
    }

    async fn needs_eviction(&self, cache: &HashMap<CacheKey, CacheEntry>, new_size: usize) -> bool {
        let current_size: usize = cache.values().map(|e| e.size_bytes).sum();
        current_size + new_size > self.config.l1_max_size
    }

    async fn evict_entries(&self, cache: &mut HashMap<CacheKey, CacheEntry>) {
        match self.config.eviction_strategy {
            EvictionStrategy::LRU => self.evict_lru(cache).await,
            EvictionStrategy::LFU => self.evict_lfu(cache).await,
            EvictionStrategy::TLRU => self.evict_tlru(cache).await,
            EvictionStrategy::Adaptive => self.evict_adaptive(cache).await,
        }
    }

    async fn evict_lru(&self, cache: &mut HashMap<CacheKey, CacheEntry>) {
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.accessed_at);
        
        // Remove oldest 20% of entries
        let remove_count = cache.len() / 5;
        let keys_to_remove: Vec<_> = entries.iter().take(remove_count).map(|(k, _)| (*k).clone()).collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }

    async fn evict_lfu(&self, cache: &mut HashMap<CacheKey, CacheEntry>) {
        let mut entries: Vec<_> = cache.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.access_count.load(Ordering::Relaxed));
        
        let remove_count = cache.len() / 5;
        let keys_to_remove: Vec<_> = entries.iter().take(remove_count).map(|(k, _)| (*k).clone()).collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }

    async fn evict_tlru(&self, cache: &mut HashMap<CacheKey, CacheEntry>) {
        let now = Instant::now();
        let mut entries: Vec<_> = cache.iter().collect();
        
        // Combine recency and frequency
        entries.sort_by_key(|(_, entry)| {
            let recency_score = now.duration_since(entry.accessed_at).as_secs();
            let frequency_score = entry.access_count.load(Ordering::Relaxed);
            recency_score * 1000 / (frequency_score + 1) // Lower is better
        });
        
        let remove_count = cache.len() / 5;
        let keys_to_remove: Vec<_> = entries.iter().take(remove_count).map(|(k, _)| (*k).clone()).collect();
        
        for key in keys_to_remove {
            cache.remove(&key);
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }

    async fn evict_adaptive(&self, cache: &mut HashMap<CacheKey, CacheEntry>) {
        // Adaptive strategy based on cache performance
        let hit_ratio = self.get_hit_ratio().await;
        
        if hit_ratio > 0.8 {
            // High hit ratio: prefer LFU to keep popular items
            self.evict_lfu(cache).await;
        } else {
            // Low hit ratio: prefer TLRU to balance recency and frequency
            self.evict_tlru(cache).await;
        }
    }

    pub async fn get_hit_ratio(&self) -> f64 {
        let total_hits = self.stats.l1_hits.load(Ordering::Relaxed) +
                        self.stats.l2_hits.load(Ordering::Relaxed) +
                        self.stats.hot_hits.load(Ordering::Relaxed);
        let total_misses = self.stats.l1_misses.load(Ordering::Relaxed) +
                          self.stats.l2_misses.load(Ordering::Relaxed);
        
        if total_hits + total_misses == 0 {
            0.0
        } else {
            total_hits as f64 / (total_hits + total_misses) as f64
        }
    }

    fn estimate_size(&self, _key: &CacheKey) -> usize {
        // Simplified size estimation
        1024 // 1KB default
    }

    /// Preload cache with predicted data
    pub async fn preload(&self, predictions: Vec<(CacheKey, CacheData)>) {
        for (key, data) in predictions {
            self.put(key, data, CachePriority::Normal).await;
        }
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        self.cleanup_cache_expired(&self.l1_cache).await;
        self.cleanup_cache_expired(&self.l2_cache).await;
        self.cleanup_cache_expired(&self.hot_cache).await;
    }

    async fn cleanup_cache_expired(&self, cache: &Arc<RwLock<HashMap<CacheKey, CacheEntry>>>) {
        let mut cache_map = cache.write().await;
        let now = Instant::now();
        
        cache_map.retain(|_, entry| {
            now.duration_since(entry.created_at) <= entry.ttl
        });
    }

    pub async fn get_detailed_stats(&self) -> CacheStatsReport {
        CacheStatsReport {
            l1_hits: self.stats.l1_hits.load(Ordering::Relaxed),
            l1_misses: self.stats.l1_misses.load(Ordering::Relaxed),
            l2_hits: self.stats.l2_hits.load(Ordering::Relaxed),
            l2_misses: self.stats.l2_misses.load(Ordering::Relaxed),
            hot_hits: self.stats.hot_hits.load(Ordering::Relaxed),
            evictions: self.stats.evictions.load(Ordering::Relaxed),
            compressions: self.stats.compressions.load(Ordering::Relaxed),
            total_size_bytes: self.stats.total_size_bytes.load(Ordering::Relaxed),
            hit_ratio: self.get_hit_ratio().await,
            l1_size: self.l1_cache.read().await.len(),
            l2_size: self.l2_cache.read().await.len(),
            hot_size: self.hot_cache.read().await.len(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStatsReport {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub hot_hits: u64,
    pub evictions: u64,
    pub compressions: u64,
    pub total_size_bytes: usize,
    pub hit_ratio: f64,
    pub l1_size: usize,
    pub l2_size: usize,
    pub hot_size: usize,
    pub timestamp: u64,
}

/// Advanced connection pooling with load balancing
pub struct AdvancedConnectionPool {
    pools: Vec<ConnectionPoolShard>,
    load_balancer: LoadBalancer,
    health_monitor: PoolHealthMonitor,
    auto_scaler: PoolAutoScaler,
}

pub struct ConnectionPoolShard {
    id: usize,
    engines: Vec<Arc<RwLock<FheEngine>>>,
    current_load: AtomicUsize,
    max_load: usize,
    health_score: AtomicU64, // 0-100
}

pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    weights: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ConsistentHashing,
    AdaptiveLoad,
}

pub struct PoolHealthMonitor {
    health_checks: VecDeque<HealthCheckResult>,
    check_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    shard_id: usize,
    timestamp: Instant,
    response_time_ms: u64,
    success: bool,
    load_percent: f64,
}

pub struct PoolAutoScaler {
    min_shards: usize,
    max_shards: usize,
    scale_up_threshold: f64,
    scale_down_threshold: f64,
    scale_up_cooldown: Duration,
    scale_down_cooldown: Duration,
    last_scale_action: Option<Instant>,
}

impl AdvancedConnectionPool {
    pub fn new(
        initial_shards: usize,
        engines_per_shard: usize,
        strategy: LoadBalancingStrategy,
    ) -> Result<Self> {
        let mut pools = Vec::with_capacity(initial_shards);
        
        for i in 0..initial_shards {
            let mut engines = Vec::with_capacity(engines_per_shard);
            for _ in 0..engines_per_shard {
                let engine = FheEngine::new(Default::default())?;
                engines.push(Arc::new(RwLock::new(engine)));
            }
            
            pools.push(ConnectionPoolShard {
                id: i,
                engines,
                current_load: AtomicUsize::new(0),
                max_load: engines_per_shard * 10, // 10 requests per engine
                health_score: AtomicU64::new(100),
            });
        }

        Ok(Self {
            pools,
            load_balancer: LoadBalancer {
                strategy,
                weights: vec![1.0; initial_shards],
            },
            health_monitor: PoolHealthMonitor {
                health_checks: VecDeque::new(),
                check_interval: Duration::from_secs(30),
            },
            auto_scaler: PoolAutoScaler {
                min_shards: 1,
                max_shards: initial_shards * 4,
                scale_up_threshold: 0.8,
                scale_down_threshold: 0.3,
                scale_up_cooldown: Duration::from_secs(300),
                scale_down_cooldown: Duration::from_secs(600),
                last_scale_action: None,
            },
        })
    }

    /// Get optimal shard for request using load balancing
    pub async fn get_optimal_shard(&self) -> usize {
        match self.load_balancer.strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin_selection(),
            LoadBalancingStrategy::LeastConnections => self.least_connections_selection(),
            LoadBalancingStrategy::WeightedRoundRobin => self.weighted_round_robin_selection(),
            LoadBalancingStrategy::ConsistentHashing => self.consistent_hash_selection(),
            LoadBalancingStrategy::AdaptiveLoad => self.adaptive_load_selection(),
        }
    }

    fn round_robin_selection(&self) -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        COUNTER.fetch_add(1, Ordering::Relaxed) % self.pools.len()
    }

    fn least_connections_selection(&self) -> usize {
        self.pools
            .iter()
            .enumerate()
            .min_by_key(|(_, shard)| shard.current_load.load(Ordering::Relaxed))
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn weighted_round_robin_selection(&self) -> usize {
        // Simplified weighted selection
        let total_weight: f64 = self.load_balancer.weights.iter().sum();
        let mut random_value = fastrand::f64() * total_weight;
        
        for (idx, &weight) in self.load_balancer.weights.iter().enumerate() {
            random_value -= weight;
            if random_value <= 0.0 {
                return idx;
            }
        }
        
        0
    }

    fn consistent_hash_selection(&self) -> usize {
        // Simplified consistent hashing based on current time
        let hash = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize;
        hash % self.pools.len()
    }

    fn adaptive_load_selection(&self) -> usize {
        // Select based on health score and current load
        self.pools
            .iter()
            .enumerate()
            .max_by_key(|(_, shard)| {
                let health = shard.health_score.load(Ordering::Relaxed);
                let load = shard.current_load.load(Ordering::Relaxed);
                let capacity = shard.max_load;
                
                // Score: health weighted by available capacity
                health * (capacity - load) as u64 / capacity as u64
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    /// Execute request with automatic load balancing
    pub async fn execute_with_balancing<F, R>(&self, operation: F) -> Result<R>
    where
        F: Fn(Arc<RwLock<FheEngine>>) -> Result<R> + Send + Sync,
        R: Send + Sync,
    {
        let shard_idx = self.get_optimal_shard().await;
        let shard = &self.pools[shard_idx];
        
        // Increment load counter
        shard.current_load.fetch_add(1, Ordering::Relaxed);
        
        let result = {
            // Select engine within shard (round-robin)
            let engine_idx = shard.current_load.load(Ordering::Relaxed) % shard.engines.len();
            let engine = Arc::clone(&shard.engines[engine_idx]);
            
            operation(engine)
        };
        
        // Decrement load counter
        shard.current_load.fetch_sub(1, Ordering::Relaxed);
        
        result
    }

    /// Monitor pool health and trigger scaling
    pub async fn monitor_and_scale(&mut self) -> Result<()> {
        // Perform health checks
        for (idx, shard) in self.pools.iter().enumerate() {
            let start = Instant::now();
            let load_percent = shard.current_load.load(Ordering::Relaxed) as f64 / shard.max_load as f64;
            
            // Simulate health check
            let success = load_percent < 0.95; // Consider unhealthy if > 95% load
            let response_time = start.elapsed().as_millis() as u64;
            
            let health_result = HealthCheckResult {
                shard_id: idx,
                timestamp: start,
                response_time_ms: response_time,
                success,
                load_percent,
            };
            
            self.health_monitor.health_checks.push_back(health_result);
            
            // Update health score
            let new_health = if success {
                std::cmp::min(100, shard.health_score.load(Ordering::Relaxed) + 5)
            } else {
                shard.health_score.load(Ordering::Relaxed).saturating_sub(10)
            };
            shard.health_score.store(new_health, Ordering::Relaxed);
        }
        
        // Trim old health checks
        while self.health_monitor.health_checks.len() > 1000 {
            self.health_monitor.health_checks.pop_front();
        }
        
        // Check if scaling is needed
        self.evaluate_scaling().await?;
        
        Ok(())
    }

    async fn evaluate_scaling(&mut self) -> Result<()> {
        let avg_load = self.get_average_load();
        let now = Instant::now();
        
        // Check cooldown
        if let Some(last_action) = self.auto_scaler.last_scale_action {
            let cooldown = if avg_load > self.auto_scaler.scale_up_threshold {
                self.auto_scaler.scale_up_cooldown
            } else {
                self.auto_scaler.scale_down_cooldown
            };
            
            if now.duration_since(last_action) < cooldown {
                return Ok(());
            }
        }
        
        // Scale up if needed
        if avg_load > self.auto_scaler.scale_up_threshold && self.pools.len() < self.auto_scaler.max_shards {
            self.scale_up().await?;
            self.auto_scaler.last_scale_action = Some(now);
            log::info!("Scaled up to {} shards due to high load: {:.2}", self.pools.len(), avg_load);
        }
        // Scale down if needed
        else if avg_load < self.auto_scaler.scale_down_threshold && self.pools.len() > self.auto_scaler.min_shards {
            self.scale_down().await?;
            self.auto_scaler.last_scale_action = Some(now);
            log::info!("Scaled down to {} shards due to low load: {:.2}", self.pools.len(), avg_load);
        }
        
        Ok(())
    }

    fn get_average_load(&self) -> f64 {
        if self.pools.is_empty() {
            return 0.0;
        }
        
        let total_load: f64 = self.pools
            .iter()
            .map(|shard| {
                shard.current_load.load(Ordering::Relaxed) as f64 / shard.max_load as f64
            })
            .sum();
        
        total_load / self.pools.len() as f64
    }

    async fn scale_up(&mut self) -> Result<()> {
        let new_shard_id = self.pools.len();
        let engines_per_shard = 4; // Default
        
        let mut engines = Vec::with_capacity(engines_per_shard);
        for _ in 0..engines_per_shard {
            let engine = FheEngine::new(Default::default())?;
            engines.push(Arc::new(RwLock::new(engine)));
        }
        
        let new_shard = ConnectionPoolShard {
            id: new_shard_id,
            engines,
            current_load: AtomicUsize::new(0),
            max_load: engines_per_shard * 10,
            health_score: AtomicU64::new(100),
        };
        
        self.pools.push(new_shard);
        self.load_balancer.weights.push(1.0);
        
        Ok(())
    }

    async fn scale_down(&mut self) -> Result<()> {
        if self.pools.len() <= self.auto_scaler.min_shards {
            return Ok(());
        }
        
        // Find the shard with the lowest load
        let remove_idx = self.pools
            .iter()
            .enumerate()
            .min_by_key(|(_, shard)| shard.current_load.load(Ordering::Relaxed))
            .map(|(idx, _)| idx)
            .unwrap_or(self.pools.len() - 1);
        
        self.pools.remove(remove_idx);
        self.load_balancer.weights.remove(remove_idx);
        
        Ok(())
    }

    pub fn get_pool_statistics(&self) -> PoolStatistics {
        let total_load = self.pools.iter()
            .map(|s| s.current_load.load(Ordering::Relaxed))
            .sum();
        
        let total_capacity = self.pools.iter()
            .map(|s| s.max_load)
            .sum();
        
        let avg_health = if !self.pools.is_empty() {
            self.pools.iter()
                .map(|s| s.health_score.load(Ordering::Relaxed))
                .sum::<u64>() / self.pools.len() as u64
        } else {
            0
        };

        PoolStatistics {
            total_shards: self.pools.len(),
            total_load,
            total_capacity,
            load_percentage: if total_capacity > 0 { 
                (total_load as f64 / total_capacity as f64) * 100.0 
            } else { 
                0.0 
            },
            average_health_score: avg_health,
            strategy: self.load_balancer.strategy.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub total_shards: usize,
    pub total_load: usize,
    pub total_capacity: usize,
    pub load_percentage: f64,
    pub average_health_score: u64,
    pub strategy: LoadBalancingStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fhe::FheParams;

    #[tokio::test]
    async fn test_performance_cache() {
        let cache = PerformanceCache::new(CacheConfig::default());
        
        let key = CacheKey {
            operation_type: "encrypt".to_string(),
            input_hash: 12345,
            params_hash: 67890,
            client_id: Uuid::new_v4(),
        };
        
        let data = CacheData::Result("test_result".to_string());
        
        // Test cache miss
        assert!(cache.get(&key).await.is_none());
        
        // Test cache put and hit
        cache.put(key.clone(), data.clone(), CachePriority::Normal).await;
        assert!(cache.get(&key).await.is_some());
        
        // Test stats
        let stats = cache.get_detailed_stats().await;
        assert!(stats.l2_hits > 0 || stats.l1_hits > 0);
    }

    #[tokio::test]
    async fn test_advanced_connection_pool() {
        let pool = AdvancedConnectionPool::new(
            2, // shards
            2, // engines per shard
            LoadBalancingStrategy::LeastConnections,
        ).unwrap();
        
        let stats = pool.get_pool_statistics();
        assert_eq!(stats.total_shards, 2);
        assert_eq!(stats.total_capacity, 40); // 2 shards * 2 engines * 10 capacity
    }

    #[test]
    fn test_cache_key_hash() {
        let key1 = CacheKey {
            operation_type: "encrypt".to_string(),
            input_hash: 12345,
            params_hash: 67890,
            client_id: Uuid::new_v4(),
        };
        
        let key2 = key1.clone();
        assert_eq!(key1, key2);
        
        let mut map = HashMap::new();
        map.insert(key1.clone(), "value1");
        assert_eq!(map.get(&key2), Some(&"value1"));
    }

    #[test]
    fn test_cache_priority_ordering() {
        assert!(CachePriority::Critical > CachePriority::High);
        assert!(CachePriority::High > CachePriority::Normal);
        assert!(CachePriority::Normal > CachePriority::Low);
    }
}