//! Naive rewrite of `hardhat-network/stack-traces/solidity-stack-traces.ts` from Hardhat.

use napi::bindgen_prelude::{
    BigInt, ClassInstance, Either24, FromNapiValue, ToNapiValue, Uint8Array, Undefined,
};
use napi_derive::napi;

use super::{model::ContractFunctionType, return_data::ReturnData};

#[napi]
#[repr(u8)]
#[allow(non_camel_case_types)] // intentionally mimicks the original case in TS
#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, PartialOrd, strum::FromRepr, strum::IntoStaticStr)]
pub enum StackTraceEntryType {
    CALLSTACK_ENTRY = 0,
    UNRECOGNIZED_CREATE_CALLSTACK_ENTRY,
    UNRECOGNIZED_CONTRACT_CALLSTACK_ENTRY,
    PRECOMPILE_ERROR,
    REVERT_ERROR,
    PANIC_ERROR,
    CUSTOM_ERROR,
    FUNCTION_NOT_PAYABLE_ERROR,
    INVALID_PARAMS_ERROR,
    FALLBACK_NOT_PAYABLE_ERROR,
    FALLBACK_NOT_PAYABLE_AND_NO_RECEIVE_ERROR,
    UNRECOGNIZED_FUNCTION_WITHOUT_FALLBACK_ERROR, // TODO: Should trying to call a private/internal be a special case of this?
    MISSING_FALLBACK_OR_RECEIVE_ERROR,
    RETURNDATA_SIZE_ERROR,
    NONCONTRACT_ACCOUNT_CALLED_ERROR,
    CALL_FAILED_ERROR,
    DIRECT_LIBRARY_CALL_ERROR,
    UNRECOGNIZED_CREATE_ERROR,
    UNRECOGNIZED_CONTRACT_ERROR,
    OTHER_EXECUTION_ERROR,
    // This is a special case to handle a regression introduced in solc 0.6.3
    // For more info: https://github.com/ethereum/solidity/issues/9006
    UNMAPPED_SOLC_0_6_3_REVERT_ERROR,
    CONTRACT_TOO_LARGE_ERROR,
    INTERNAL_FUNCTION_CALLSTACK_ENTRY,
    CONTRACT_CALL_RUN_OUT_OF_GAS_ERROR,
}

#[napi]
pub fn stack_trace_entry_type_to_string(val: StackTraceEntryType) -> &'static str {
    val.into()
}

#[napi]
pub const FALLBACK_FUNCTION_NAME: &str = "<fallback>";
#[napi]
pub const RECEIVE_FUNCTION_NAME: &str = "<receive>";
#[napi]
pub const CONSTRUCTOR_FUNCTION_NAME: &str = "constructor";
#[napi]
pub const UNRECOGNIZED_FUNCTION_NAME: &str = "<unrecognized-selector>";
#[napi]
pub const UNKNOWN_FUNCTION_NAME: &str = "<unknown>";
#[napi]
pub const PRECOMPILE_FUNCTION_NAME: &str = "<precompile>";
#[napi]
pub const UNRECOGNIZED_CONTRACT_NAME: &str = "<UnrecognizedContract>";

#[napi(object)]
#[derive(Clone, PartialEq)]
pub struct SourceReference {
    pub source_name: String,
    pub source_content: String,
    pub contract: Option<String>,
    pub function: Option<String>,
    pub line: u32,
    // [number, number] tuple
    pub range: Vec<u32>,
}

/// A [`StackTraceEntryType`] constant that is convertible to/from a `napi_value`.
///
/// Since Rust does not allow constants directly as members, we use this wrapper
/// to allow the `StackTraceEntryType` to be used as a member of an interface
/// when defining the N-API bindings.
// NOTE: It's currently not possible to use an enum as const generic parameter,
// so we use the underlying `u8` repr used by the enum.
#[derive(Clone, Copy)]
pub struct StackTraceEntryTypeConst<const ENTRY_TYPE: u8>;
impl<const ENTRY_TYPE: u8> FromNapiValue for StackTraceEntryTypeConst<ENTRY_TYPE> {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        napi_val: napi::sys::napi_value,
    ) -> napi::Result<Self> {
        let inner: u8 = FromNapiValue::from_napi_value(env, napi_val)?;

        if inner != ENTRY_TYPE {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                format!("Expected StackTraceEntryType value: {ENTRY_TYPE}, got: {inner}"),
            ));
        }

        Ok(StackTraceEntryTypeConst)
    }
}
impl<const ENTRY_TYPE: u8> ToNapiValue for StackTraceEntryTypeConst<ENTRY_TYPE> {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        _val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        u8::to_napi_value(env, ENTRY_TYPE)
    }
}

impl<const ENTRY_TYPE: u8> StackTraceEntryTypeConst<ENTRY_TYPE> {
    #[allow(clippy::unused_self)] // less verbose than <value as ...>::as_value()
    const fn as_value(&self) -> StackTraceEntryType {
        match StackTraceEntryType::from_repr(ENTRY_TYPE) {
            Some(val) => val,
            None => panic!("Invalid StackTraceEntryType value"),
        }
    }
}

#[napi(object)]
pub struct CallstackEntryStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.CALLSTACK_ENTRY")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::CALLSTACK_ENTRY as u8 }>,
    pub source_reference: SourceReference,
    pub function_type: ContractFunctionType,
}

#[napi(object)]
pub struct UnrecognizedCreateCallstackEntryStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.UNRECOGNIZED_CREATE_CALLSTACK_ENTRY"
    )]
    pub type_: StackTraceEntryTypeConst<
        { StackTraceEntryType::UNRECOGNIZED_CREATE_CALLSTACK_ENTRY as u8 },
    >,
    pub source_reference: Option<Undefined>,
}

#[napi(object)]
pub struct UnrecognizedContractCallstackEntryStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.UNRECOGNIZED_CONTRACT_CALLSTACK_ENTRY"
    )]
    pub type_: StackTraceEntryTypeConst<
        { StackTraceEntryType::UNRECOGNIZED_CONTRACT_CALLSTACK_ENTRY as u8 },
    >,
    pub address: Uint8Array,
    pub source_reference: Option<Undefined>,
}

#[napi(object)]
pub struct PrecompileErrorStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.PRECOMPILE_ERROR")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::PRECOMPILE_ERROR as u8 }>,
    pub precompile: u32,
    pub source_reference: Option<Undefined>,
}

#[napi(object)]
pub struct RevertErrorStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.REVERT_ERROR")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::REVERT_ERROR as u8 }>,
    pub message: ClassInstance<ReturnData>,
    pub source_reference: SourceReference,
    pub is_invalid_opcode_error: bool,
}

#[napi(object)]
pub struct PanicErrorStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.PANIC_ERROR")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::PANIC_ERROR as u8 }>,
    pub error_code: BigInt,
    pub source_reference: Option<SourceReference>,
}

#[napi(object)]
pub struct CustomErrorStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.CUSTOM_ERROR")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::CUSTOM_ERROR as u8 }>,
    // unlike RevertErrorStackTraceEntry, this includes the message already parsed
    pub message: String,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct UnmappedSolc063RevertErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.UNMAPPED_SOLC_0_6_3_REVERT_ERROR"
    )]
    pub type_:
        StackTraceEntryTypeConst<{ StackTraceEntryType::UNMAPPED_SOLC_0_6_3_REVERT_ERROR as u8 }>,
    pub source_reference: Option<SourceReference>,
}

#[napi(object)]
pub struct FunctionNotPayableErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.FUNCTION_NOT_PAYABLE_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::FUNCTION_NOT_PAYABLE_ERROR as u8 }>,
    pub value: BigInt,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct InvalidParamsErrorStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.INVALID_PARAMS_ERROR")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::INVALID_PARAMS_ERROR as u8 }>,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct FallbackNotPayableErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.FALLBACK_NOT_PAYABLE_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::FALLBACK_NOT_PAYABLE_ERROR as u8 }>,
    pub value: BigInt,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct FallbackNotPayableAndNoReceiveErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.FALLBACK_NOT_PAYABLE_AND_NO_RECEIVE_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<
        { StackTraceEntryType::FALLBACK_NOT_PAYABLE_AND_NO_RECEIVE_ERROR as u8 },
    >,
    pub value: BigInt,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct UnrecognizedFunctionWithoutFallbackErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.UNRECOGNIZED_FUNCTION_WITHOUT_FALLBACK_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<
        { StackTraceEntryType::UNRECOGNIZED_FUNCTION_WITHOUT_FALLBACK_ERROR as u8 },
    >,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct MissingFallbackOrReceiveErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.MISSING_FALLBACK_OR_RECEIVE_ERROR"
    )]
    pub type_:
        StackTraceEntryTypeConst<{ StackTraceEntryType::MISSING_FALLBACK_OR_RECEIVE_ERROR as u8 }>,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct ReturndataSizeErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.RETURNDATA_SIZE_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::RETURNDATA_SIZE_ERROR as u8 }>,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct NonContractAccountCalledErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.NONCONTRACT_ACCOUNT_CALLED_ERROR"
    )]
    pub type_:
        StackTraceEntryTypeConst<{ StackTraceEntryType::NONCONTRACT_ACCOUNT_CALLED_ERROR as u8 }>,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct CallFailedErrorStackTraceEntry {
    #[napi(js_name = "type", ts_type = "StackTraceEntryType.CALL_FAILED_ERROR")]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::CALL_FAILED_ERROR as u8 }>,
    pub source_reference: SourceReference,
}
#[napi(object)]
pub struct DirectLibraryCallErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.DIRECT_LIBRARY_CALL_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::DIRECT_LIBRARY_CALL_ERROR as u8 }>,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct UnrecognizedCreateErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.UNRECOGNIZED_CREATE_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::UNRECOGNIZED_CREATE_ERROR as u8 }>,
    #[napi(ts_type = "ReturnData")]
    pub message: ClassInstance<ReturnData>,
    pub source_reference: Option<Undefined>,
    pub is_invalid_opcode_error: bool,
}

#[napi(object)]
pub struct UnrecognizedContractErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.UNRECOGNIZED_CONTRACT_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::UNRECOGNIZED_CONTRACT_ERROR as u8 }>,
    pub address: Uint8Array,
    pub message: ClassInstance<ReturnData>,
    pub source_reference: Option<Undefined>,
    pub is_invalid_opcode_error: bool,
}

#[napi(object)]
pub struct OtherExecutionErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.OTHER_EXECUTION_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::OTHER_EXECUTION_ERROR as u8 }>,
    pub source_reference: Option<SourceReference>,
}

#[napi(object)]
pub struct ContractTooLargeErrorStackTraceEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.CONTRACT_TOO_LARGE_ERROR"
    )]
    pub type_: StackTraceEntryTypeConst<{ StackTraceEntryType::CONTRACT_TOO_LARGE_ERROR as u8 }>,
    pub source_reference: Option<SourceReference>,
}

#[napi(object)]
pub struct InternalFunctionCallStackEntry {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.INTERNAL_FUNCTION_CALLSTACK_ENTRY"
    )]
    pub type_:
        StackTraceEntryTypeConst<{ StackTraceEntryType::INTERNAL_FUNCTION_CALLSTACK_ENTRY as u8 }>,
    pub pc: u32,
    pub source_reference: SourceReference,
}

#[napi(object)]
pub struct ContractCallRunOutOfGasError {
    #[napi(
        js_name = "type",
        ts_type = "StackTraceEntryType.CONTRACT_CALL_RUN_OUT_OF_GAS_ERROR"
    )]
    pub type_:
        StackTraceEntryTypeConst<{ StackTraceEntryType::CONTRACT_CALL_RUN_OUT_OF_GAS_ERROR as u8 }>,
    pub source_reference: Option<SourceReference>,
}

// NOTE: This ported directly from JS for completeness, however the type must be
// used verbatim in JS definitions because napi-rs does not store not allows to
// reuse the same type unless fully specified at definition site.
pub type SolidityStackTraceEntry = Either24<
    CallstackEntryStackTraceEntry,
    UnrecognizedCreateCallstackEntryStackTraceEntry,
    UnrecognizedContractCallstackEntryStackTraceEntry,
    PrecompileErrorStackTraceEntry,
    RevertErrorStackTraceEntry,
    PanicErrorStackTraceEntry,
    CustomErrorStackTraceEntry,
    FunctionNotPayableErrorStackTraceEntry,
    InvalidParamsErrorStackTraceEntry,
    FallbackNotPayableErrorStackTraceEntry,
    FallbackNotPayableAndNoReceiveErrorStackTraceEntry,
    UnrecognizedFunctionWithoutFallbackErrorStackTraceEntry,
    MissingFallbackOrReceiveErrorStackTraceEntry,
    ReturndataSizeErrorStackTraceEntry,
    NonContractAccountCalledErrorStackTraceEntry,
    CallFailedErrorStackTraceEntry,
    DirectLibraryCallErrorStackTraceEntry,
    UnrecognizedCreateErrorStackTraceEntry,
    UnrecognizedContractErrorStackTraceEntry,
    OtherExecutionErrorStackTraceEntry,
    UnmappedSolc063RevertErrorStackTraceEntry,
    ContractTooLargeErrorStackTraceEntry,
    InternalFunctionCallStackEntry,
    ContractCallRunOutOfGasError,
>;

pub type SolidityStackTrace = Vec<SolidityStackTraceEntry>;

pub trait SolidityStackTraceEntryExt {
    fn type_(&self) -> StackTraceEntryType;
    fn source_reference(&self) -> Option<&SourceReference>;
}

impl SolidityStackTraceEntryExt for SolidityStackTraceEntry {
    fn type_(&self) -> StackTraceEntryType {
        match self {
            Either24::A(entry) => entry.type_.as_value(),
            Either24::B(entry) => entry.type_.as_value(),
            Either24::C(entry) => entry.type_.as_value(),
            Either24::D(entry) => entry.type_.as_value(),
            Either24::E(entry) => entry.type_.as_value(),
            Either24::F(entry) => entry.type_.as_value(),
            Either24::G(entry) => entry.type_.as_value(),
            Either24::H(entry) => entry.type_.as_value(),
            Either24::I(entry) => entry.type_.as_value(),
            Either24::J(entry) => entry.type_.as_value(),
            Either24::K(entry) => entry.type_.as_value(),
            Either24::L(entry) => entry.type_.as_value(),
            Either24::M(entry) => entry.type_.as_value(),
            Either24::N(entry) => entry.type_.as_value(),
            Either24::O(entry) => entry.type_.as_value(),
            Either24::P(entry) => entry.type_.as_value(),
            Either24::Q(entry) => entry.type_.as_value(),
            Either24::R(entry) => entry.type_.as_value(),
            Either24::S(entry) => entry.type_.as_value(),
            Either24::T(entry) => entry.type_.as_value(),
            Either24::U(entry) => entry.type_.as_value(),
            Either24::V(entry) => entry.type_.as_value(),
            Either24::W(entry) => entry.type_.as_value(),
            Either24::X(entry) => entry.type_.as_value(),
        }
    }

    #[allow(clippy::unnecessary_lazy_evaluations)] // guards against potential variant reordering
    fn source_reference(&self) -> Option<&SourceReference> {
        match self {
            Either24::A(entry) => Some(&entry.source_reference),
            Either24::B(entry) => entry.source_reference.and_then(|_: ()| None),
            Either24::C(entry) => entry.source_reference.and_then(|_: ()| None),
            Either24::D(entry) => entry.source_reference.and_then(|_: ()| None),
            Either24::E(entry) => Some(&entry.source_reference),
            Either24::F(entry) => entry.source_reference.as_ref(),
            Either24::G(entry) => Some(&entry.source_reference),
            Either24::H(entry) => Some(&entry.source_reference),
            Either24::I(entry) => Some(&entry.source_reference),
            Either24::J(entry) => Some(&entry.source_reference),
            Either24::K(entry) => Some(&entry.source_reference),
            Either24::L(entry) => Some(&entry.source_reference),
            Either24::M(entry) => Some(&entry.source_reference),
            Either24::N(entry) => Some(&entry.source_reference),
            Either24::O(entry) => Some(&entry.source_reference),
            Either24::P(entry) => Some(&entry.source_reference),
            Either24::Q(entry) => Some(&entry.source_reference),
            Either24::R(entry) => entry.source_reference.and_then(|_: ()| None),
            Either24::S(entry) => entry.source_reference.and_then(|_: ()| None),
            Either24::T(entry) => entry.source_reference.as_ref(),
            Either24::U(entry) => entry.source_reference.as_ref(),
            Either24::V(entry) => entry.source_reference.as_ref(),
            Either24::W(entry) => Some(&entry.source_reference),
            Either24::X(entry) => entry.source_reference.as_ref(),
        }
    }
}

const _: () = {
    const fn assert_to_from_napi_value<T: FromNapiValue + ToNapiValue>() {}
    assert_to_from_napi_value::<SolidityStackTraceEntry>();
};