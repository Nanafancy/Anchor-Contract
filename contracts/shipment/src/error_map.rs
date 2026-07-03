use crate::errors::AnchorError;

/// Broad category a contract error belongs to.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorCategory {
    /// Caller supplied bad input (wrong hash, invalid amount, etc.).
    InvalidInput,
    /// Caller lacks the required role or signature.
    Unauthorized,
    /// The requested resource does not exist.
    NotFound,
    /// The operation is not allowed in the current state.
    InvalidState,
    /// A resource limit or rate cap was hit.
    LimitExceeded,
    /// A transient infrastructure or arithmetic failure.
    Transient,
    /// Contract-level configuration or initialisation problem.
    Configuration,
}

/// Retry posture the caller should adopt after receiving this error.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RetryGuidance {
    /// Do not retry; fix the request before resubmitting.
    NoRetry,
    /// Retry after a short delay (network / rate-limit transient).
    RetryAfterDelay,
    /// Retry only after the on-chain state changes (e.g. wait for expiry).
    RetryAfterStateChange,
}

/// Structured metadata for a single `AnchorError` variant.
#[derive(Copy, Clone, Debug)]
pub struct ContractErrorInfo {
    pub error: AnchorError,
    /// Numeric discriminant as exposed on-chain.
    pub code: u32,
    pub category: ErrorCategory,
    pub retry: RetryGuidance,
    /// Short human-readable description suitable for operator logs / UI.
    pub message: &'static str,
}

/// Returns the `ContractErrorInfo` for the given `AnchorError`.
///
/// Consumers (backends, frontends, indexers) call this to translate a raw
/// contract error code into a category and retry decision without hard-coding
/// the mapping themselves.
///
/// # Example
/// ```rust
/// use shipment::error_map::{error_info, RetryGuidance};
/// use shipment::errors::AnchorError;
///
/// let info = error_info(AnchorError::RateLimitExceeded);
/// assert_eq!(info.retry, RetryGuidance::RetryAfterDelay);
/// ```
pub fn error_info(error: AnchorError) -> ContractErrorInfo {
    use ErrorCategory::*;
    use RetryGuidance::*;

    let (code, category, retry, message) = match error {
        AnchorError::AlreadyInitialized => (
            1,
            Configuration,
            NoRetry,
            "Contract is already initialised; call init only once.",
        ),
        AnchorError::NotInitialized => (
            2,
            Configuration,
            NoRetry,
            "Contract has not been initialised; call init first.",
        ),
        AnchorError::Unauthorized => (
            3,
            Unauthorized,
            NoRetry,
            "Caller does not hold the required role or signature.",
        ),
        AnchorError::ShipmentNotFound => (4, NotFound, NoRetry, "Shipment ID does not exist."),
        AnchorError::InvalidStatus => (
            5,
            InvalidState,
            RetryAfterStateChange,
            "State transition is not allowed from the current shipment status.",
        ),
        AnchorError::InvalidHash => (
            6,
            InvalidInput,
            NoRetry,
            "Provided data hash does not match the stored value.",
        ),
        AnchorError::EscrowLocked => (
            7,
            InvalidState,
            RetryAfterStateChange,
            "Escrow is locked; wait for the shipment to reach a terminal state.",
        ),
        AnchorError::InsufficientFunds => (
            8,
            InvalidInput,
            NoRetry,
            "Caller balance is too low to cover the escrow deposit.",
        ),
        AnchorError::ShipmentAlreadyCompleted => (
            9,
            InvalidState,
            NoRetry,
            "Shipment is already in a terminal state (Delivered or Disputed).",
        ),
        AnchorError::InvalidTimestamp => (
            10,
            InvalidInput,
            NoRetry,
            "Timestamp is invalid (e.g. ETA is in the past).",
        ),
        AnchorError::CounterOverflow => (
            11,
            Transient,
            NoRetry,
            "Internal counter overflowed; contact the contract operator.",
        ),
        AnchorError::InvalidAmount => (
            14,
            InvalidInput,
            NoRetry,
            "Amount must be a positive non-zero value.",
        ),
        AnchorError::ReentrancyDetected => (
            15,
            InvalidState,
            RetryAfterDelay,
            "Reentrancy lock is active; retry once the current escrow operation completes.",
        ),
        AnchorError::BatchTooLarge => (
            16,
            LimitExceeded,
            NoRetry,
            "Batch exceeds the maximum allowed item count; split into smaller batches.",
        ),
        AnchorError::InvalidShipmentInput => (
            17,
            InvalidInput,
            NoRetry,
            "Shipment parameters are invalid (e.g. receiver equals carrier).",
        ),
        AnchorError::MilestoneSumInvalid => (
            18,
            InvalidInput,
            NoRetry,
            "Milestone percentages must sum to exactly 100.",
        ),
        AnchorError::MilestoneAlreadyPaid => (
            19,
            InvalidState,
            NoRetry,
            "This milestone has already been paid.",
        ),
        AnchorError::MetadataLimitExceeded => (
            20,
            LimitExceeded,
            NoRetry,
            "Maximum of 5 metadata entries per shipment reached.",
        ),
        AnchorError::RateLimitExceeded => (
            21,
            LimitExceeded,
            RetryAfterDelay,
            "Minimum interval between status updates has not elapsed; retry later.",
        ),
        AnchorError::ProposalNotFound => (
            22,
            NotFound,
            NoRetry,
            "Multi-sig proposal ID does not exist.",
        ),
        AnchorError::ProposalAlreadyExecuted => (
            23,
            InvalidState,
            NoRetry,
            "Proposal has already been executed.",
        ),
        AnchorError::ProposalExpired => (
            24,
            InvalidState,
            NoRetry,
            "Proposal has expired; create a new proposal.",
        ),
        AnchorError::AlreadyApproved => (
            25,
            InvalidState,
            NoRetry,
            "This admin has already approved the proposal.",
        ),
        AnchorError::InsufficientApprovals => (
            26,
            InvalidState,
            RetryAfterStateChange,
            "Not enough admin approvals; wait for additional signers.",
        ),
        AnchorError::NotAnAdmin => (
            27,
            Unauthorized,
            NoRetry,
            "Caller is not in the admin list.",
        ),
        AnchorError::InvalidMultiSigConfig => (
            28,
            InvalidInput,
            NoRetry,
            "Multi-sig config is invalid (e.g. threshold exceeds admin count).",
        ),
        AnchorError::NotExpired => (
            29,
            InvalidState,
            RetryAfterStateChange,
            "Shipment deadline has not yet passed; wait for expiry.",
        ),
        AnchorError::ShipmentLimitReached => (
            30,
            LimitExceeded,
            RetryAfterStateChange,
            "Company has reached its active shipment cap; close existing shipments first.",
        ),
        AnchorError::InvalidConfig => (
            31,
            InvalidInput,
            NoRetry,
            "Configuration parameters are invalid.",
        ),
        AnchorError::CannotSelfRevoke => (
            32,
            InvalidInput,
            NoRetry,
            "An admin cannot revoke their own role; use transfer_admin instead.",
        ),
        AnchorError::CarrierSuspended => (
            33,
            Unauthorized,
            RetryAfterStateChange,
            "Carrier account is suspended; contact the contract operator.",
        ),
        AnchorError::ForceCancelReasonHashMissing => (
            34,
            InvalidInput,
            NoRetry,
            "Force-cancel requires a non-zero reason hash.",
        ),
        AnchorError::ArithmeticError => (
            35,
            Transient,
            NoRetry,
            "Arithmetic overflow/underflow in escrow calculation; check amounts.",
        ),
        AnchorError::DisputeReasonHashMissing => (
            36,
            InvalidInput,
            NoRetry,
            "Dispute resolution requires a non-zero reason hash.",
        ),
        AnchorError::CompanySuspended => (
            37,
            Unauthorized,
            RetryAfterStateChange,
            "Company account is suspended; contact the contract operator.",
        ),
        AnchorError::ShipmentFinalized => (
            38,
            InvalidState,
            NoRetry,
            "Shipment is finalised and locked; no further mutations are allowed.",
        ),
        AnchorError::TokenTransferFailed => (
            39,
            Transient,
            RetryAfterDelay,
            "Cross-contract token transfer failed; retry after verifying token contract state.",
        ),
        AnchorError::TokenMintFailed => (
            40,
            Transient,
            RetryAfterDelay,
            "Cross-contract token mint failed; retry after verifying token contract state.",
        ),
        AnchorError::DuplicateAction => (
            41,
            InvalidInput,
            NoRetry,
            "Action hash was already processed within the idempotency window.",
        ),
        AnchorError::ShipmentUnavailable => (
            42,
            InvalidState,
            RetryAfterStateChange,
            "Shipment state is unavailable (archived or expired); restore before retrying.",
        ),
        AnchorError::ContractPaused => (
            43,
            InvalidState,
            RetryAfterStateChange,
            "Contract is paused; wait for the operator to resume operations.",
        ),
        AnchorError::StatusHashNotFound => (
            44,
            NotFound,
            NoRetry,
            "No status hash found for the given shipment and status.",
        ),
        AnchorError::DataHashMismatch => (
            45,
            InvalidInput,
            NoRetry,
            "Provided hash does not match the stored hash; recompute and resubmit.",
        ),
        AnchorError::CircuitBreakerOpen => (
            46,
            Transient,
            RetryAfterDelay,
            "Circuit breaker is open; token transfers are temporarily disabled.",
        ),
        AnchorError::InvalidMigrationEdge => (
            47,
            InvalidInput,
            NoRetry,
            "Migration version transition is not permitted.",
        ),
        AnchorError::MilestoneLimitExceeded => (
            48,
            LimitExceeded,
            NoRetry,
            "Maximum milestone events per shipment reached.",
        ),
        AnchorError::NoteLimitExceeded => (
            49,
            LimitExceeded,
            NoRetry,
            "Maximum note events per shipment reached.",
        ),
        AnchorError::EvidenceLimitExceeded => (
            50,
            LimitExceeded,
            NoRetry,
            "Maximum evidence entries per dispute reached.",
        ),
        AnchorError::BreachLimitExceeded => (
            51,
            LimitExceeded,
            NoRetry,
            "Maximum condition breach events per shipment reached.",
        ),
        AnchorError::InvalidTokenDecimals => (
            52,
            InvalidInput,
            NoRetry,
            "Token decimals do not match the expected value (7); use a Stellar-standard token.",
        ),
        AnchorError::CreationQuotaExceeded => (
            53,
            LimitExceeded,
            RetryAfterStateChange,
            "Company has exceeded the shipment creation quota for the current time window.",
        ),
        AnchorError::DependenciesNotMet => (
            54,
            InvalidState,
            RetryAfterStateChange,
            "Shipment cannot transition to InTransit or Delivered because its prerequisite shipments are not yet completed.",
        ),
        AnchorError::CircularDependency => (
            55,
            InvalidInput,
            NoRetry,
            "A circular dependency was detected in the shipment prerequisites.",
        ),
        AnchorError::ProposalSaltReused => (
            56,
            InvalidInput,
            NoRetry,
            "Proposal salt was already used in a prior proposal; replay attack prevented.",
        ),
        AnchorError::InvalidShipmentParticipants => (
            57,
            InvalidInput,
            NoRetry,
            "Shipment sender, receiver, and carrier must be three distinct addresses.",
        ),
        AnchorError::InvalidShipmentDeadline => (
            58,
            InvalidInput,
            NoRetry,
            "Shipment deadline must be strictly in the future.",
        ),
        AnchorError::InvalidPaymentMilestones => (
            59,
            InvalidInput,
            NoRetry,
            "Payment milestone structure is invalid; each percentage must be 1-100.",
        ),
        AnchorError::DuplicatePaymentMilestone => (
            60,
            InvalidInput,
            NoRetry,
            "Payment milestone checkpoint names must be unique.",
        ),
        AnchorError::InvalidTokenAddress => (
            61,
            InvalidInput,
            NoRetry,
            "Shipment token address is invalid for this shipment.",
        ),
        AnchorError::InvalidPaymentMilestoneName => (
            62,
            InvalidInput,
            NoRetry,
            "Payment milestone checkpoint name has an invalid format.",
        ),
        AnchorError::MetadataSymbolCollision => (
            63,
            InvalidInput,
            NoRetry,
            "Metadata key and value symbols are identical; use distinct symbols.",
        ),
    };

    ContractErrorInfo {
        error,
        code,
        category,
        retry,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AnchorError;

    // ── Token transfer failure recovery — error mapping (issue #447) ─────────

    #[test]
    fn test_token_transfer_failed_info() {
        let info = error_info(AnchorError::TokenTransferFailed);
        assert_eq!(info.error, AnchorError::TokenTransferFailed);
        assert_eq!(info.code, 39);
        assert_eq!(info.category, ErrorCategory::Transient);
        assert_eq!(info.retry, RetryGuidance::RetryAfterDelay);
        assert!(
            !info.message.is_empty(),
            "TokenTransferFailed must have a non-empty message"
        );
    }

    #[test]
    fn test_circuit_breaker_open_info() {
        let info = error_info(AnchorError::CircuitBreakerOpen);
        assert_eq!(info.error, AnchorError::CircuitBreakerOpen);
        assert_eq!(info.code, 46);
        assert_eq!(info.category, ErrorCategory::Transient);
        assert_eq!(info.retry, RetryGuidance::RetryAfterDelay);
    }

    /// error_info must be deterministic — calling it twice on the same variant
    /// must return identical results.
    #[test]
    fn test_error_info_is_deterministic() {
        let a = error_info(AnchorError::TokenTransferFailed);
        let b = error_info(AnchorError::TokenTransferFailed);
        assert_eq!(a.code, b.code);
        assert_eq!(a.category, b.category);
        assert_eq!(a.retry, b.retry);
        assert_eq!(a.message, b.message);

        let c = error_info(AnchorError::CircuitBreakerOpen);
        let d = error_info(AnchorError::CircuitBreakerOpen);
        assert_eq!(c.code, d.code);
        assert_eq!(c.category, d.category);
        assert_eq!(c.retry, d.retry);
    }

    /// Token-related transient errors must use RetryAfterDelay, not NoRetry,
    /// so callers know they can retry after a backoff.
    #[test]
    fn test_token_and_circuit_breaker_errors_use_retry_after_delay() {
        let transient_errors = [
            AnchorError::TokenTransferFailed,
            AnchorError::TokenMintFailed,
            AnchorError::CircuitBreakerOpen,
        ];
        for err in &transient_errors {
            let info = error_info(*err);
            assert_eq!(
                info.retry,
                RetryGuidance::RetryAfterDelay,
                "{:?} must have RetryAfterDelay guidance",
                err
            );
            assert_eq!(
                info.category,
                ErrorCategory::Transient,
                "{:?} must be categorised as Transient",
                err
            );
        }
    }

    /// Every error code in error_info must match its AnchorError discriminant.
    #[test]
    fn test_error_codes_match_discriminants() {
        let cases: &[(AnchorError, u32)] = &[
            (AnchorError::TokenTransferFailed, 39),
            (AnchorError::TokenMintFailed, 40),
            (AnchorError::CircuitBreakerOpen, 46),
            (AnchorError::ShipmentFinalized, 38),
            (AnchorError::ShipmentNotFound, 4),
            (AnchorError::Unauthorized, 3),
        ];
        for (err, expected_code) in cases {
            let info = error_info(*err);
            assert_eq!(
                info.code, *expected_code,
                "{:?} must map to code {}",
                err, expected_code
            );
        }
    }
}
