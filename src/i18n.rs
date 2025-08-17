//! Internationalization support for FHE LLM Proxy

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Chinese => "zh",
            Language::Japanese => "ja",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            "fr" => Some(Language::French),
            "de" => Some(Language::German),
            "zh" => Some(Language::Chinese),
            "ja" => Some(Language::Japanese),
            _ => None,
        }
    }

    pub fn from_accept_language(accept_language: &str) -> Self {
        let languages = accept_language
            .split(',')
            .map(|lang| lang.split(';').next().unwrap_or("").trim())
            .collect::<Vec<_>>();

        for lang in languages {
            if let Some(language) = Self::from_code(lang) {
                return language;
            }
            // Handle language variants (e.g., "en-US" -> "en")
            if let Some(base_lang) = lang.split('-').next() {
                if let Some(language) = Self::from_code(base_lang) {
                    return language;
                }
            }
        }

        Language::English // Default fallback
    }
}

/// Translation structure for nested JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Translations {
    pub app: AppTranslations,
    pub errors: ErrorTranslations,
    pub messages: MessageTranslations,
    pub api: ApiTranslations,
    pub compliance: ComplianceTranslations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTranslations {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTranslations {
    pub validation: String,
    pub encryption: String,
    pub auth: String,
    pub rate_limit: String,
    pub privacy_budget: String,
    pub config: String,
    pub network: String,
    pub internal: String,
    pub security: String,
    pub resource_exhaustion: String,
    pub concurrency: String,
    pub data_corruption: String,
    pub cryptographic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTranslations {
    pub startup: String,
    pub shutdown: String,
    pub key_rotation: String,
    pub cache_eviction: String,
    pub health_check: String,
    pub privacy_budget_reset: String,
    pub encryption_complete: String,
    pub decryption_complete: String,
    pub validation_passed: String,
    pub validation_failed: String,
    pub metrics_recorded: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTranslations {
    pub health_endpoint: String,
    pub metrics_endpoint: String,
    pub encrypt_endpoint: String,
    pub decrypt_endpoint: String,
    pub process_endpoint: String,
    pub keys_endpoint: String,
    pub params_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTranslations {
    pub gdpr: GdprTranslations,
    pub ccpa: CcpaTranslations,
    pub hipaa: HipaaTranslations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprTranslations {
    pub enabled: String,
    pub data_subject_rights: String,
    pub consent_required: String,
    pub retention_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcpaTranslations {
    pub enabled: String,
    pub privacy_rights: String,
    pub opt_out: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HipaaTranslations {
    pub enabled: String,
    pub phi_protection: String,
    pub audit_logging: String,
}

/// Internationalization manager
#[derive(Debug, Clone)]
pub struct I18n {
    translations: HashMap<Language, Translations>,
    default_language: Language,
}

impl I18n {
    /// Create new I18n instance with default language
    pub fn new(default_language: Language) -> Self {
        Self {
            translations: HashMap::new(),
            default_language,
        }
    }

    /// Load translations from locale directory
    pub fn load_translations<P: AsRef<Path>>(mut self, locale_dir: P) -> crate::error::Result<Self> {
        let locale_path = locale_dir.as_ref();
        
        for &language in &[
            Language::English,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Chinese,
            Language::Japanese,
        ] {
            let file_path = locale_path.join(format!("{}.json", language.code()));
            
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)
                    .map_err(|e| crate::error::Error::Config(format!("Failed to read locale file {}: {}", file_path.display(), e)))?;
                
                let translations: Translations = serde_json::from_str(&content)
                    .map_err(|e| crate::error::Error::Config(format!("Failed to parse locale file {}: {}", file_path.display(), e)))?;
                
                self.translations.insert(language, translations);
                
                log::info!("Loaded translations for language: {}", language.code());
            } else {
                log::warn!("Translation file not found: {}", file_path.display());
            }
        }

        if self.translations.is_empty() {
            return Err(crate::error::Error::Config("No translation files loaded".to_string()));
        }

        log::info!("Internationalization initialized with {} languages", self.translations.len());
        Ok(self)
    }

    /// Get translations for a specific language
    pub fn get_translations(&self, language: Language) -> Option<&Translations> {
        self.translations.get(&language)
    }

    /// Get translations with fallback to default language
    pub fn get_translations_with_fallback(&self, language: Language) -> &Translations {
        self.translations
            .get(&language)
            .or_else(|| self.translations.get(&self.default_language))
            .expect("Default language translations must be available")
    }

    /// Translate a specific error message
    pub fn translate_error(&self, language: Language, error_key: &str) -> String {
        let translations = self.get_translations_with_fallback(language);
        
        match error_key {
            "validation" => translations.errors.validation.clone(),
            "encryption" => translations.errors.encryption.clone(),
            "auth" => translations.errors.auth.clone(),
            "rate_limit" => translations.errors.rate_limit.clone(),
            "privacy_budget" => translations.errors.privacy_budget.clone(),
            "config" => translations.errors.config.clone(),
            "network" => translations.errors.network.clone(),
            "internal" => translations.errors.internal.clone(),
            "security" => translations.errors.security.clone(),
            "resource_exhaustion" => translations.errors.resource_exhaustion.clone(),
            "concurrency" => translations.errors.concurrency.clone(),
            "data_corruption" => translations.errors.data_corruption.clone(),
            "cryptographic" => translations.errors.cryptographic.clone(),
            _ => format!("Unknown error: {}", error_key),
        }
    }

    /// Translate a specific message
    pub fn translate_message(&self, language: Language, message_key: &str) -> String {
        let translations = self.get_translations_with_fallback(language);
        
        match message_key {
            "startup" => translations.messages.startup.clone(),
            "shutdown" => translations.messages.shutdown.clone(),
            "key_rotation" => translations.messages.key_rotation.clone(),
            "cache_eviction" => translations.messages.cache_eviction.clone(),
            "health_check" => translations.messages.health_check.clone(),
            "privacy_budget_reset" => translations.messages.privacy_budget_reset.clone(),
            "encryption_complete" => translations.messages.encryption_complete.clone(),
            "decryption_complete" => translations.messages.decryption_complete.clone(),
            "validation_passed" => translations.messages.validation_passed.clone(),
            "validation_failed" => translations.messages.validation_failed.clone(),
            "metrics_recorded" => translations.messages.metrics_recorded.clone(),
            _ => format!("Unknown message: {}", message_key),
        }
    }

    /// Get app name in specified language
    pub fn get_app_name(&self, language: Language) -> String {
        self.get_translations_with_fallback(language).app.name.clone()
    }

    /// Get app description in specified language
    pub fn get_app_description(&self, language: Language) -> String {
        self.get_translations_with_fallback(language).app.description.clone()
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.translations.keys().cloned().collect()
    }

    /// Check if a language is supported
    pub fn is_supported(&self, language: Language) -> bool {
        self.translations.contains_key(&language)
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new(Language::English)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Spanish.code(), "es");
        assert_eq!(Language::French.code(), "fr");
        assert_eq!(Language::German.code(), "de");
        assert_eq!(Language::Chinese.code(), "zh");
        assert_eq!(Language::Japanese.code(), "ja");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("es"), Some(Language::Spanish));
        assert_eq!(Language::from_code("invalid"), None);
    }

    #[test]
    fn test_accept_language_parsing() {
        assert_eq!(
            Language::from_accept_language("en-US,en;q=0.9,es;q=0.8"),
            Language::English
        );
        assert_eq!(
            Language::from_accept_language("es-ES,es;q=0.9"),
            Language::Spanish
        );
        assert_eq!(
            Language::from_accept_language("fr-FR,fr;q=0.9,en;q=0.8"),
            Language::French
        );
        assert_eq!(
            Language::from_accept_language("invalid"),
            Language::English
        );
    }

    #[test]
    fn test_i18n_creation() {
        let i18n = I18n::new(Language::English);
        assert_eq!(i18n.default_language, Language::English);
        assert!(i18n.translations.is_empty());
    }
}