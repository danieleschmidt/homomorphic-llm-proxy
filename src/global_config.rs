//! Global Configuration System
//!
//! Comprehensive global-first configuration supporting:
//! - Multi-region deployment
//! - GDPR/CCPA/PDPA compliance
//! - Cross-platform compatibility
//! - Currency and timezone support
//! - Regional data sovereignty

use crate::error::{Error, Result};
use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Global configuration for multi-region deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfiguration {
    /// Regional configurations
    pub regions: HashMap<Region, RegionalConfig>,
    /// Default region for new deployments
    pub default_region: Region,
    /// Compliance configurations
    pub compliance: ComplianceConfig,
    /// Localization settings
    pub localization: LocalizationConfig,
    /// Currency support
    pub currencies: CurrencyConfig,
    /// Data sovereignty rules
    pub data_sovereignty: DataSovereigntyConfig,
}

/// Supported deployment regions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    /// North America
    NorthAmerica,
    /// Europe (GDPR compliant)
    Europe,
    /// Asia Pacific
    AsiaPacific,
    /// South America
    SouthAmerica,
    /// Middle East & Africa
    MiddleEastAfrica,
    /// China (with specific regulatory compliance)
    China,
}

impl Region {
    /// Get region code for configuration
    pub fn code(&self) -> &'static str {
        match self {
            Region::NorthAmerica => "na",
            Region::Europe => "eu",
            Region::AsiaPacific => "ap",
            Region::SouthAmerica => "sa",
            Region::MiddleEastAfrica => "mea",
            Region::China => "cn",
        }
    }

    /// Get regulatory jurisdiction
    pub fn jurisdiction(&self) -> RegulatoryJurisdiction {
        match self {
            Region::NorthAmerica => RegulatoryJurisdiction::CCPA,
            Region::Europe => RegulatoryJurisdiction::GDPR,
            Region::AsiaPacific => RegulatoryJurisdiction::PDPA,
            Region::SouthAmerica => RegulatoryJurisdiction::LGPD,
            Region::MiddleEastAfrica => RegulatoryJurisdiction::GDPR,
            Region::China => RegulatoryJurisdiction::PIPL,
        }
    }

    /// Get primary languages for the region
    pub fn primary_languages(&self) -> Vec<Language> {
        match self {
            Region::NorthAmerica => vec![Language::English, Language::Spanish],
            Region::Europe => vec![Language::English, Language::German, Language::French],
            Region::AsiaPacific => vec![Language::English, Language::Japanese, Language::Chinese],
            Region::SouthAmerica => vec![Language::Spanish, Language::English],
            Region::MiddleEastAfrica => vec![Language::English, Language::French],
            Region::China => vec![Language::Chinese, Language::English],
        }
    }
}

/// Regional configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalConfig {
    /// Data center locations
    pub data_centers: Vec<DataCenter>,
    /// Regional compliance requirements
    pub compliance_level: ComplianceLevel,
    /// Maximum data retention period
    pub max_retention_days: u32,
    /// Local currency
    pub primary_currency: Currency,
    /// Business hours (for support)
    pub business_hours: BusinessHours,
    /// Regional API endpoints
    pub api_endpoints: ApiEndpoints,
    /// Performance optimization settings
    pub performance_settings: RegionalPerformanceSettings,
}

/// Data center configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCenter {
    /// Data center ID
    pub id: String,
    /// Geographic location
    pub location: GeographicLocation,
    /// Availability zones
    pub availability_zones: Vec<String>,
    /// Network latency SLA (milliseconds)
    pub latency_sla_ms: u32,
    /// Uptime SLA (percentage)
    pub uptime_sla_percent: f64,
    /// Backup data centers
    pub backup_centers: Vec<String>,
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicLocation {
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
}

/// Regulatory jurisdictions and compliance frameworks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegulatoryJurisdiction {
    /// General Data Protection Regulation (Europe)
    GDPR,
    /// California Consumer Privacy Act (US)
    CCPA,
    /// Personal Data Protection Act (Singapore/APAC)
    PDPA,
    /// Lei Geral de Proteção de Dados (Brazil)
    LGPD,
    /// Personal Information Protection Law (China)
    PIPL,
}

/// Compliance level requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Basic compliance
    Basic,
    /// Standard business compliance
    Standard,
    /// High security/financial compliance
    High,
    /// Government/military grade
    Maximum,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enabled jurisdictions
    pub jurisdictions: Vec<RegulatoryJurisdiction>,
    /// Data subject rights configuration
    pub data_subject_rights: DataSubjectRights,
    /// Audit logging requirements
    pub audit_requirements: AuditRequirements,
    /// Data minimization settings
    pub data_minimization: DataMinimizationConfig,
    /// Consent management
    pub consent_management: ConsentManagementConfig,
}

/// Data subject rights (GDPR Article 12-23)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRights {
    /// Right to access (Article 15)
    pub right_to_access: bool,
    /// Right to rectification (Article 16)
    pub right_to_rectification: bool,
    /// Right to erasure/"right to be forgotten" (Article 17)
    pub right_to_erasure: bool,
    /// Right to restrict processing (Article 18)
    pub right_to_restrict_processing: bool,
    /// Right to data portability (Article 20)
    pub right_to_data_portability: bool,
    /// Right to object (Article 21)
    pub right_to_object: bool,
    /// Response time requirement (days)
    pub response_time_days: u32,
}

/// Audit requirements for compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRequirements {
    /// Enable comprehensive audit logging
    pub enabled: bool,
    /// Log retention period (days)
    pub retention_days: u32,
    /// Required audit events
    pub required_events: Vec<AuditEvent>,
    /// Log integrity verification
    pub integrity_verification: bool,
    /// External audit compliance
    pub external_audit_ready: bool,
}

/// Types of events that must be audited
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditEvent {
    DataAccess,
    DataModification,
    DataDeletion,
    UserAuthentication,
    PrivilegeEscalation,
    SystemConfiguration,
    SecurityIncident,
    ConsentChanges,
    DataTransfer,
    BackupRestore,
}

/// Data minimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMinimizationConfig {
    /// Automatic data purging enabled
    pub auto_purge_enabled: bool,
    /// Default retention periods by data type
    pub retention_policies: HashMap<DataType, u32>,
    /// Data classification requirements
    pub classification_required: bool,
    /// Purpose limitation enforcement
    pub purpose_limitation: bool,
}

/// Data types for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    PersonalData,
    SensitivePersonalData,
    FinancialData,
    HealthData,
    BiometricData,
    LocationData,
    CommunicationData,
    SystemLogs,
    AnalyticsData,
    BackupData,
}

/// Consent management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentManagementConfig {
    /// Granular consent required
    pub granular_consent: bool,
    /// Consent withdrawal mechanism
    pub withdrawal_enabled: bool,
    /// Consent expiration period (days)
    pub consent_expiry_days: Option<u32>,
    /// Double opt-in required
    pub double_opt_in: bool,
    /// Consent audit trail
    pub audit_trail: bool,
}

/// Localization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationConfig {
    /// Supported languages
    pub supported_languages: Vec<Language>,
    /// Default fallback language
    pub default_language: Language,
    /// RTL language support
    pub rtl_support: bool,
    /// Date/time formats by region
    pub datetime_formats: HashMap<Region, DateTimeFormat>,
    /// Number formats by region
    pub number_formats: HashMap<Region, NumberFormat>,
    /// Address formats by country
    pub address_formats: HashMap<String, AddressFormat>,
}

/// Date and time formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateTimeFormat {
    pub date_format: String,
    pub time_format: String,
    pub timezone_display: bool,
    pub week_start: WeekDay,
}

/// Number formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberFormat {
    pub decimal_separator: String,
    pub thousands_separator: String,
    pub currency_symbol_position: CurrencyPosition,
}

/// Address formatting by country
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressFormat {
    pub fields: Vec<AddressField>,
    pub postal_code_format: String,
    pub validation_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeekDay {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyPosition {
    Before,
    After,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressField {
    StreetAddress,
    City,
    StateProvince,
    PostalCode,
    Country,
    ApartmentUnit,
    Neighborhood,
    Prefecture,
}

/// Currency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConfig {
    /// Supported currencies
    pub supported_currencies: Vec<Currency>,
    /// Default currency
    pub default_currency: Currency,
    /// Exchange rate provider
    pub exchange_rate_provider: ExchangeRateProvider,
    /// Rate update frequency
    pub update_frequency: Duration,
    /// Pricing by region
    pub regional_pricing: HashMap<Region, RegionalPricing>,
}

/// Supported currencies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    USD, // US Dollar
    EUR, // Euro
    GBP, // British Pound
    JPY, // Japanese Yen
    CNY, // Chinese Yuan
    CAD, // Canadian Dollar
    AUD, // Australian Dollar
    CHF, // Swiss Franc
    KRW, // South Korean Won
    SGD, // Singapore Dollar
    BRL, // Brazilian Real
    INR, // Indian Rupee
}

impl Currency {
    /// Get currency code
    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::CNY => "CNY",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
            Currency::CHF => "CHF",
            Currency::KRW => "KRW",
            Currency::SGD => "SGD",
            Currency::BRL => "BRL",
            Currency::INR => "INR",
        }
    }

    /// Get currency symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
            Currency::JPY => "¥",
            Currency::CNY => "¥",
            Currency::CAD => "C$",
            Currency::AUD => "A$",
            Currency::CHF => "Fr",
            Currency::KRW => "₩",
            Currency::SGD => "S$",
            Currency::BRL => "R$",
            Currency::INR => "₹",
        }
    }
}

/// Exchange rate providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeRateProvider {
    ECB,        // European Central Bank
    OpenExchangeRates,
    CurrencyLayer,
    Fixer,
}

/// Regional pricing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalPricing {
    pub base_currency: Currency,
    pub price_adjustments: HashMap<String, f64>, // service -> multiplier
    pub tax_rates: HashMap<String, f64>, // tax type -> rate
    pub discount_programs: Vec<DiscountProgram>,
}

/// Discount programs by region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountProgram {
    pub name: String,
    pub discount_percent: f64,
    pub eligibility_criteria: String,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Data sovereignty configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSovereigntyConfig {
    /// Cross-border data transfer rules
    pub transfer_rules: HashMap<Region, TransferRule>,
    /// Data residency requirements
    pub residency_requirements: HashMap<Region, ResidencyRequirement>,
    /// Encryption requirements for transfer
    pub transfer_encryption: EncryptionRequirement,
    /// Government access policies
    pub government_access: GovernmentAccessPolicy,
}

/// Cross-border data transfer rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRule {
    /// Allowed destination regions
    pub allowed_destinations: Vec<Region>,
    /// Required adequacy decisions
    pub adequacy_decisions: Vec<String>,
    /// Standard contractual clauses required
    pub scc_required: bool,
    /// Binding corporate rules
    pub bcr_required: bool,
    /// Additional safeguards
    pub additional_safeguards: Vec<String>,
}

/// Data residency requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyRequirement {
    /// Data must remain in region
    pub strict_residency: bool,
    /// Allowed processing regions
    pub processing_regions: Vec<Region>,
    /// Backup/DR regions allowed
    pub backup_regions: Vec<Region>,
    /// Mirror data requirements
    pub mirroring_required: bool,
}

/// Encryption requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequirement {
    /// Minimum encryption level
    pub minimum_level: EncryptionLevel,
    /// Key management requirements
    pub key_management: KeyManagementRequirement,
    /// End-to-end encryption required
    pub e2e_required: bool,
    /// Hardware security modules required
    pub hsm_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncryptionLevel {
    AES128,
    AES256,
    FHE,        // Fully Homomorphic Encryption
    PostQuantum, // Post-quantum cryptography
}

/// Key management requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementRequirement {
    /// Regional key storage
    pub regional_keys: bool,
    /// Key escrow allowed
    pub escrow_allowed: bool,
    /// Key rotation period (days)
    pub rotation_period_days: u32,
    /// Multi-party key management
    pub multi_party: bool,
}

/// Government access policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentAccessPolicy {
    /// Warrant requirements
    pub warrant_required: bool,
    /// Notification requirements
    pub notification_required: bool,
    /// Challenge process available
    pub challenge_available: bool,
    /// Transparency reporting
    pub transparency_reporting: bool,
}

/// Business hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHours {
    pub timezone: String,
    pub monday: Option<TimeRange>,
    pub tuesday: Option<TimeRange>,
    pub wednesday: Option<TimeRange>,
    pub thursday: Option<TimeRange>,
    pub friday: Option<TimeRange>,
    pub saturday: Option<TimeRange>,
    pub sunday: Option<TimeRange>,
    pub holidays: Vec<Holiday>,
}

/// Time range for business hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: String, // HH:MM format
    pub end: String,   // HH:MM format
}

/// Holiday configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holiday {
    pub name: String,
    pub date: String, // YYYY-MM-DD or recurring pattern
    pub recurring: bool,
}

/// Regional API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoints {
    pub primary: String,
    pub backup: Vec<String>,
    pub cdn_endpoints: HashMap<String, String>, // service -> endpoint
    pub websocket_endpoints: Vec<String>,
}

/// Regional performance optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalPerformanceSettings {
    /// CDN configuration
    pub cdn_config: CdnConfig,
    /// Caching strategy
    pub cache_strategy: CacheStrategy,
    /// Connection pooling
    pub connection_pooling: ConnectionPooling,
    /// Request batching
    pub request_batching: RequestBatching,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    pub provider: String,
    pub cache_ttl_seconds: u32,
    pub edge_locations: Vec<String>,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategy {
    pub strategy_type: String,
    pub ttl_seconds: u32,
    pub max_size_mb: u32,
    pub regional_caching: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPooling {
    pub max_connections: u32,
    pub idle_timeout_seconds: u32,
    pub connection_reuse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBatching {
    pub enabled: bool,
    pub max_batch_size: u32,
    pub batch_timeout_ms: u32,
}

impl GlobalConfiguration {
    /// Create default global configuration
    pub fn default() -> Self {
        let mut regions = HashMap::new();
        
        // Configure North America
        regions.insert(Region::NorthAmerica, RegionalConfig {
            data_centers: vec![
                DataCenter {
                    id: "us-east-1".to_string(),
                    location: GeographicLocation {
                        country: "United States".to_string(),
                        region: "Virginia".to_string(),
                        city: "Ashburn".to_string(),
                        latitude: 39.0458,
                        longitude: -77.5019,
                        timezone: "America/New_York".to_string(),
                    },
                    availability_zones: vec!["us-east-1a".to_string(), "us-east-1b".to_string(), "us-east-1c".to_string()],
                    latency_sla_ms: 100,
                    uptime_sla_percent: 99.99,
                    backup_centers: vec!["us-west-2".to_string()],
                },
            ],
            compliance_level: ComplianceLevel::High,
            max_retention_days: 2555, // 7 years for financial data
            primary_currency: Currency::USD,
            business_hours: BusinessHours {
                timezone: "America/New_York".to_string(),
                monday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                tuesday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                wednesday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                thursday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                friday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                saturday: None,
                sunday: None,
                holidays: vec![],
            },
            api_endpoints: ApiEndpoints {
                primary: "https://api-na.fhe-llm.com".to_string(),
                backup: vec!["https://api-backup-na.fhe-llm.com".to_string()],
                cdn_endpoints: HashMap::new(),
                websocket_endpoints: vec!["wss://ws-na.fhe-llm.com".to_string()],
            },
            performance_settings: RegionalPerformanceSettings {
                cdn_config: CdnConfig {
                    provider: "CloudFlare".to_string(),
                    cache_ttl_seconds: 3600,
                    edge_locations: vec!["US".to_string(), "CA".to_string()],
                    compression_enabled: true,
                },
                cache_strategy: CacheStrategy {
                    strategy_type: "LRU".to_string(),
                    ttl_seconds: 1800,
                    max_size_mb: 512,
                    regional_caching: true,
                },
                connection_pooling: ConnectionPooling {
                    max_connections: 100,
                    idle_timeout_seconds: 300,
                    connection_reuse: true,
                },
                request_batching: RequestBatching {
                    enabled: true,
                    max_batch_size: 32,
                    batch_timeout_ms: 50,
                },
            },
        });

        // Configure Europe (GDPR)
        regions.insert(Region::Europe, RegionalConfig {
            data_centers: vec![
                DataCenter {
                    id: "eu-west-1".to_string(),
                    location: GeographicLocation {
                        country: "Ireland".to_string(),
                        region: "Dublin".to_string(),
                        city: "Dublin".to_string(),
                        latitude: 53.3498,
                        longitude: -6.2603,
                        timezone: "Europe/Dublin".to_string(),
                    },
                    availability_zones: vec!["eu-west-1a".to_string(), "eu-west-1b".to_string(), "eu-west-1c".to_string()],
                    latency_sla_ms: 100,
                    uptime_sla_percent: 99.99,
                    backup_centers: vec!["eu-central-1".to_string()],
                },
            ],
            compliance_level: ComplianceLevel::Maximum, // GDPR requires maximum compliance
            max_retention_days: 1095, // 3 years default, varies by data type
            primary_currency: Currency::EUR,
            business_hours: BusinessHours {
                timezone: "Europe/Dublin".to_string(),
                monday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                tuesday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                wednesday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                thursday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                friday: Some(TimeRange { start: "09:00".to_string(), end: "17:00".to_string() }),
                saturday: None,
                sunday: None,
                holidays: vec![],
            },
            api_endpoints: ApiEndpoints {
                primary: "https://api-eu.fhe-llm.com".to_string(),
                backup: vec!["https://api-backup-eu.fhe-llm.com".to_string()],
                cdn_endpoints: HashMap::new(),
                websocket_endpoints: vec!["wss://ws-eu.fhe-llm.com".to_string()],
            },
            performance_settings: RegionalPerformanceSettings {
                cdn_config: CdnConfig {
                    provider: "CloudFlare".to_string(),
                    cache_ttl_seconds: 3600,
                    edge_locations: vec!["EU".to_string()],
                    compression_enabled: true,
                },
                cache_strategy: CacheStrategy {
                    strategy_type: "LRU".to_string(),
                    ttl_seconds: 1800,
                    max_size_mb: 512,
                    regional_caching: true,
                },
                connection_pooling: ConnectionPooling {
                    max_connections: 100,
                    idle_timeout_seconds: 300,
                    connection_reuse: true,
                },
                request_batching: RequestBatching {
                    enabled: true,
                    max_batch_size: 32,
                    batch_timeout_ms: 50,
                },
            },
        });

        Self {
            regions,
            default_region: Region::NorthAmerica,
            compliance: ComplianceConfig {
                jurisdictions: vec![
                    RegulatoryJurisdiction::GDPR,
                    RegulatoryJurisdiction::CCPA,
                    RegulatoryJurisdiction::PDPA,
                ],
                data_subject_rights: DataSubjectRights {
                    right_to_access: true,
                    right_to_rectification: true,
                    right_to_erasure: true,
                    right_to_restrict_processing: true,
                    right_to_data_portability: true,
                    right_to_object: true,
                    response_time_days: 30,
                },
                audit_requirements: AuditRequirements {
                    enabled: true,
                    retention_days: 2555, // 7 years
                    required_events: vec![
                        AuditEvent::DataAccess,
                        AuditEvent::DataModification,
                        AuditEvent::DataDeletion,
                        AuditEvent::UserAuthentication,
                        AuditEvent::ConsentChanges,
                        AuditEvent::DataTransfer,
                    ],
                    integrity_verification: true,
                    external_audit_ready: true,
                },
                data_minimization: DataMinimizationConfig {
                    auto_purge_enabled: true,
                    retention_policies: {
                        let mut policies = HashMap::new();
                        policies.insert(DataType::PersonalData, 1095);
                        policies.insert(DataType::SensitivePersonalData, 365);
                        policies.insert(DataType::SystemLogs, 365);
                        policies.insert(DataType::AnalyticsData, 730);
                        policies
                    },
                    classification_required: true,
                    purpose_limitation: true,
                },
                consent_management: ConsentManagementConfig {
                    granular_consent: true,
                    withdrawal_enabled: true,
                    consent_expiry_days: Some(730), // 2 years
                    double_opt_in: true,
                    audit_trail: true,
                },
            },
            localization: LocalizationConfig {
                supported_languages: vec![
                    Language::English,
                    Language::Spanish,
                    Language::French,
                    Language::German,
                    Language::Chinese,
                    Language::Japanese,
                ],
                default_language: Language::English,
                rtl_support: false, // Could be extended for Arabic/Hebrew
                datetime_formats: {
                    let mut formats = HashMap::new();
                    formats.insert(Region::NorthAmerica, DateTimeFormat {
                        date_format: "MM/dd/yyyy".to_string(),
                        time_format: "h:mm a".to_string(),
                        timezone_display: true,
                        week_start: WeekDay::Sunday,
                    });
                    formats.insert(Region::Europe, DateTimeFormat {
                        date_format: "dd/MM/yyyy".to_string(),
                        time_format: "HH:mm".to_string(),
                        timezone_display: true,
                        week_start: WeekDay::Monday,
                    });
                    formats
                },
                number_formats: {
                    let mut formats = HashMap::new();
                    formats.insert(Region::NorthAmerica, NumberFormat {
                        decimal_separator: ".".to_string(),
                        thousands_separator: ",".to_string(),
                        currency_symbol_position: CurrencyPosition::Before,
                    });
                    formats.insert(Region::Europe, NumberFormat {
                        decimal_separator: ",".to_string(),
                        thousands_separator: ".".to_string(),
                        currency_symbol_position: CurrencyPosition::After,
                    });
                    formats
                },
                address_formats: HashMap::new(),
            },
            currencies: CurrencyConfig {
                supported_currencies: vec![
                    Currency::USD,
                    Currency::EUR,
                    Currency::GBP,
                    Currency::JPY,
                    Currency::CNY,
                    Currency::CAD,
                    Currency::AUD,
                ],
                default_currency: Currency::USD,
                exchange_rate_provider: ExchangeRateProvider::ECB,
                update_frequency: Duration::from_secs(3600), // 1 hour
                regional_pricing: HashMap::new(),
            },
            data_sovereignty: DataSovereigntyConfig {
                transfer_rules: HashMap::new(),
                residency_requirements: HashMap::new(),
                transfer_encryption: EncryptionRequirement {
                    minimum_level: EncryptionLevel::FHE, // Use FHE for maximum privacy
                    key_management: KeyManagementRequirement {
                        regional_keys: true,
                        escrow_allowed: false,
                        rotation_period_days: 90,
                        multi_party: true,
                    },
                    e2e_required: true,
                    hsm_required: true,
                },
                government_access: GovernmentAccessPolicy {
                    warrant_required: true,
                    notification_required: true,
                    challenge_available: true,
                    transparency_reporting: true,
                },
            },
        }
    }

    /// Get configuration for a specific region
    pub fn get_regional_config(&self, region: &Region) -> Option<&RegionalConfig> {
        self.regions.get(region)
    }

    /// Validate compliance requirements for a region
    pub fn validate_compliance(&self, region: &Region) -> Result<()> {
        let jurisdiction = region.jurisdiction();
        
        if !self.compliance.jurisdictions.contains(&jurisdiction) {
            return Err(Error::Configuration(
                format!("Jurisdiction {:?} not supported in current compliance configuration", jurisdiction)
            ));
        }

        // Validate GDPR-specific requirements
        if jurisdiction == RegulatoryJurisdiction::GDPR {
            if !self.compliance.data_subject_rights.right_to_erasure {
                return Err(Error::Configuration(
                    "GDPR requires right to erasure to be enabled".to_string()
                ));
            }
            
            if self.compliance.data_subject_rights.response_time_days > 30 {
                return Err(Error::Configuration(
                    "GDPR requires response time of 30 days or less".to_string()
                ));
            }
        }

        log::info!("Compliance validation passed for region {:?}", region);
        Ok(())
    }

    /// Get supported languages for a region
    pub fn get_supported_languages(&self, region: &Region) -> Vec<Language> {
        region.primary_languages()
            .into_iter()
            .filter(|lang| self.localization.supported_languages.contains(lang))
            .collect()
    }

    /// Check if cross-border data transfer is allowed
    pub fn is_transfer_allowed(&self, from: &Region, to: &Region) -> bool {
        if let Some(rule) = self.data_sovereignty.transfer_rules.get(from) {
            rule.allowed_destinations.contains(to)
        } else {
            // Default: allow within same jurisdiction
            from.jurisdiction() == to.jurisdiction()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_creation() {
        let config = GlobalConfiguration::default();
        assert_eq!(config.default_region, Region::NorthAmerica);
        assert!(config.regions.contains_key(&Region::NorthAmerica));
        assert!(config.regions.contains_key(&Region::Europe));
    }

    #[test]
    fn test_gdpr_compliance_validation() {
        let config = GlobalConfiguration::default();
        let result = config.validate_compliance(&Region::Europe);
        assert!(result.is_ok());
    }

    #[test]
    fn test_language_support() {
        let config = GlobalConfiguration::default();
        let na_languages = config.get_supported_languages(&Region::NorthAmerica);
        assert!(na_languages.contains(&Language::English));
        assert!(na_languages.contains(&Language::Spanish));
    }

    #[test]
    fn test_currency_codes() {
        assert_eq!(Currency::USD.code(), "USD");
        assert_eq!(Currency::EUR.code(), "EUR");
        assert_eq!(Currency::USD.symbol(), "$");
        assert_eq!(Currency::EUR.symbol(), "€");
    }

    #[test]
    fn test_region_jurisdiction() {
        assert_eq!(Region::Europe.jurisdiction(), RegulatoryJurisdiction::GDPR);
        assert_eq!(Region::NorthAmerica.jurisdiction(), RegulatoryJurisdiction::CCPA);
        assert_eq!(Region::AsiaPacific.jurisdiction(), RegulatoryJurisdiction::PDPA);
    }

    #[test]
    fn test_data_transfer_rules() {
        let config = GlobalConfiguration::default();
        
        // Same jurisdiction should be allowed
        assert!(config.is_transfer_allowed(&Region::NorthAmerica, &Region::NorthAmerica));
        
        // Default behavior: different jurisdictions require explicit rules
        assert!(!config.is_transfer_allowed(&Region::Europe, &Region::China));
    }
}