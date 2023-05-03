# Changelog

## v0.14.2 (October 26, 2022)

- Derived impls of `FromMeta` will now error on literals, rather than silently ignoring them. [#193](https://github.com/TedDriggs/darling/pull/193)
- Don't include property paths in compile errors when spans are available. [#203](https://github.com/TedDriggs/darling/pull/203)

## v0.14.1 (April 28, 2022)

- Fix a bug where using a trait that accepts `#[darling(attributes(...))]` without specifying any attributes would emit code that did not compile. [#183](https://github.com/TedDriggs/darling/issues/183)
- Impl `Clone` for `darling::Error` [#184](https://github.com/TedDriggs/darling/pull/184)
- Impl `From<darling::Error> for syn::Error` [#184](https://github.com/TedDriggs/darling/pull/184)
- Add `Error::span` and `Error::explicit_span` methods [#184](https://github.com/TedDriggs/darling/pull/184)

## v0.14.0 (April 13, 2022)

- **BREAKING CHANGE:** Remove many trait impls from `util::Flag`. 
  This type had a number of deref and operator impls that made it usable as sort-of-a-boolean.
  Real-world usage showed this type is more useful if it's able to carry a span for good errors,
  and that most of those impls were unnecessary. [#179](https://github.com/TedDriggs/darling/pull/179)
- Remove need for `#[darling(default)]` on `Option<T>` and `Flag` fields [#161](https://github.com/TedDriggs/darling/issues/161)
- Improve validation of enum shapes [#178](https://github.com/TedDriggs/darling/pull/178)
- Bump `proc-macro2` dependency to 1.0.37 [#180](https://github.com/TedDriggs/darling/pull/180)
- Bump `quote` dependency to 1.0.18 [#180](https://github.com/TedDriggs/darling/pull/180)
- Bump `syn` dependency to 1.0.91 [#180](https://github.com/TedDriggs/darling/pull/180)

## v0.13.4 (April 6, 2022)

- Impl `FromMeta` for `syn::Visibility` [#173](https://github.com/TedDriggs/darling/pull/173)

## v0.13.3 (April 5, 2022)

- Add `error::Accumulator` for dealing with multiple errors [#164](https://github.com/TedDriggs/darling/pull/164)
- Impl `FromMeta` for `syn::Type` and its variants [#172](https://github.com/TedDriggs/darling/pulls/172)

## v0.13.2 (March 30, 2022)

- Impl `FromMeta` for `syn::ExprPath` [#169](https://github.com/TedDriggs/darling/issues/169)

## v0.13.1 (December 7, 2021)

- Add `FromAttributes` trait and macro [#151](https://github.com/TedDriggs/darling/issues/151)

## v0.13.0 (May 20, 2021)

- Update darling to 2018 edition [#129](https://github.com/TedDriggs/darling/pull/129)
- Error on duplicate fields in `#[darling(...)]` attributes [#130](https://github.com/TedDriggs/darling/pull/130)
- Impl `Copy` for `SpannedValue<T: Copy>`
- Add `SpannedValue::map_ref`

## v0.13.0-beta (April 20, 2021)

- Update darling to 2018 edition [#129](https://github.com/TedDriggs/darling/pull/129)
- Error on duplicate fields in `#[darling(...)]` attributes [#130](https://github.com/TedDriggs/darling/pull/130)

## v0.12.4 (April 20, 2021)

- Add `and_then` to derive macros for `darling`

## v0.12.3 (April 8, 2021)

- Fix `FromMeta` impl for `char` not to panic [#126](https://github.com/TedDriggs/darling/pull/126)

## v0.12.2 (February 23, 2021)

- Impl `FromMeta` for `HashMap<Ident, V>` and `HashMap<Path, V>`

## v0.12.1 (February 22, 2021)

- Impl `FromMeta` for `syn::ExprArray` [#122](https://github.com/TedDriggs/darling/pull/122)
- Remove use of `unreachable` from `darling::ast::Data` [#123](https://github.com/TedDriggs/darling/pull/123)
- Add `darling::ast::Data::try_empty_from` to avoid panics when trying to read a union body [#123](https://github.com/TedDriggs/darling/pull/123)

## v0.12.0 (January 5, 2021)

- POSSIBLY BREAKING: Derived impls of `FromDeriveInput`, `FromField`, `FromVariant`, and `FromTypeParam` will now error when encountering an attribute `darling` has been asked to parse that isn't a supported shape.
  Any crates using `darling` that relied on those attributes being silently ignored could see new errors reported in their dependent crates. [#113](https://github.com/TedDriggs/darling/pull/113)
- Impl `syn::spanned::Spanned` for `darling::util::SpannedValue` [#113](https://github.com/TedDriggs/darling/pull/113)
- Add `darling::util::parse_attribute_to_meta_list` to provide useful errors during attribute parsing [#113](https://github.com/TedDriggs/darling/pull/113)
- Add `impl From<syn::Error> for Error` to losslessly propagate `syn` errors [#116](https://github.com/TedDriggs/darling/pull/116)

## v0.11.0 (December 14, 2020)

- Bump minor version due to unexpected breaking change [#107](https://github.com/TedDriggs/darling/issues/107)

## v0.10.3 (December 10, 2020)

- Add `discriminant` magic field when deriving `FromVariant` [#105](https://github.com/TedDriggs/darling/pull/105)

## v0.10.2 (October 30, 2019)

- Bump syn dependency to 1.0.1 [#83](https://github.com/TedDriggs/darling/pull/83)

## v0.10.1 (September 25, 2019)

- Fix test compilation errors [#81](https://github.com/TedDriggs/darling/pull/81)

## v0.10.0 (August 15, 2019)

- Bump syn and quote to 1.0 [#79](https://github.com/TedDriggs/darling/pull/79)
- Increase rust version to 1.31

## v0.9.0 (March 20, 2019)

- Enable "did you mean" suggestions by default
- Make `darling_core::{codegen, options}` private [#58](https://github.com/TedDriggs/darling/issues/58)
- Fix `Override::as_mut`: [#66](https://github.com/TedDriggs/darling/issues/66)

## v0.8.6 (March 18, 2019)

- Added "did you mean" suggestions for unknown fields behind the `suggestions` flag [#60](https://github.com/TedDriggs/issues/60)
- Added `Error::unknown_field_with_alts` to support the suggestion use-case.
- Added `ast::Fields::len` and `ast::Fields::is_empty` methods.

## v0.8.5 (February 4, 2019)

- Accept unquoted positive numeric literals [#52](https://github.com/TedDriggs/issues/52)
- Add `FromMeta` to the `syn::Lit` enum and its variants
- Improve error message for unexpected literal formats to not say "other"

## v0.8.4 (February 4, 2019)

- Use `syn::Error` to provide precise errors before `proc_macro::Diagnostic` is available
- Add `diagnostics` feature flag to toggle between stable and unstable error backends
- Attach error information in more contexts
- Add `allow_unknown_fields` to support parsing the same attribute multiple times for different macros [#51](https://github.com/darling/issues/51)
- Proc-macro authors will now see better errors in `darling` attributes

## v0.8.3 (January 21, 2019)

- Attach spans to errors in generated trait impls [#37](https://github.com/darling/issues/37)
- Attach spans to errors for types with provided bespoke implementations
- Deprecate `set_span` from 0.8.2, as spans should never be broadened after being initially set

## v0.8.2 (January 17, 2019)

- Add spans to errors to make quality warnings and errors easy in darling. This is blocked on diagnostics stabilizing.
- Add `darling::util::SpannedValue` so proc-macro authors can remember position information alongside parsed values.

## v0.8.0

- Update dependency on `syn` to 0.15 [#44](https://github.com/darling/pull/44). Thanks to @hcpl

## v0.7.0 (July 24, 2018)

- Update dependencies on `syn` and `proc-macro2`
- Add `util::IdentString`, which acts as an Ident or its string equivalent

## v0.6.3 (May 22, 2018)

- Add support for `Uses*` traits in where predicates

## v0.6.2 (May 22, 2018)

- Add `usage` module for tracking type param and lifetime usage in generic declarations
  - Add `UsesTypeParams` and `CollectsTypeParams` traits [#37](https://github.com/darling/issues/37)
  - Add `UsesLifetimes` and `CollectLifetimes` traits [#41](https://github.com/darling/pull/41)
- Don't add `FromMeta` bounds to type parameters only used by skipped fields [#40](https://github.com/darling/pull/40)

## v0.6.1 (May 17, 2018)

- Fix an issue where the `syn` update broke shape validation [#36](https://github.com/TedDriggs/darling/issues/36)

## v0.6.0 (May 15, 2018)

### Breaking Changes

- Renamed `FromMetaItem` to `FromMeta`, and renamed `from_meta_item` method to `from_meta`
- Added dedicated `derive(FromMetaItem)` which panics and redirects users to `FromMeta`

## v0.5.0 (May 10, 2018)

- Add `ast::Generics` and `ast::GenericParam` to work with generics in a manner similar to `ast::Data`
- Add `ast::GenericParamExt` to support alternate representations of generic parameters
- Add `util::WithOriginal` to get a parsed representation and syn's own struct for a syntax block
- Add `FromGenerics` and `FromGenericParam` traits (without derive support)
- Change generated code for `generics` magic field to invoke `FromGenerics` trait during parsing
- Add `FromTypeParam` trait [#30](https://github.com/TedDriggs/darling/pull/30). Thanks to @upsuper

## v0.4.0 (April 5, 2018)

- Update dependencies on `proc-macro`, `quote`, and `syn` [#26](https://github.com/TedDriggs/darling/pull/26). Thanks to @hcpl

## v0.3.3 (April 2, 2018)

**YANKED**

## v0.3.2 (March 13, 2018)

- Derive `Default` on `darling::Ignored` (fixes [#25](https://github.com/TedDriggs/darling/issues/25)).

## v0.3.1 (March 7, 2018)

- Support proc-macro2/nightly [#24](https://github.com/TedDriggs/darling/pull/24). Thanks to @kdy1

## v0.3.0 (January 26, 2018)

### Breaking Changes

- Update `syn` to 0.12 [#20](https://github.com/TedDriggs/darling/pull/20). Thanks to @Eijebong
- Update `quote` to 0.4 [#20](https://github.com/TedDriggs/darling/pull/20). Thanks to @Eijebong
- Rename magic field `body` in derived `FromDeriveInput` structs to `data` to stay in sync with `syn`
- Rename magic field `data` in derived `FromVariant` structs to `fields` to stay in sync with `syn`

## v0.2.2 (December 5, 2017)

- Update `lazy_static` to 1.0 [#15](https://github.com/TedDriggs/darling/pull/16). Thanks to @Eijebong

## v0.2.1 (November 28, 2017)

- Add `impl FromMetaItem` for integer types [#15](https://github.com/TedDriggs/darling/pull/15)

## v0.2.0 (June 18, 2017)

- Added support for returning multiple errors from parsing [#5](https://github.com/TedDriggs/darling/pull/5)
- Derived impls no longer return on first error [#5](https://github.com/TedDriggs/darling/pull/5)
- Removed default types for `V` and `F` from `ast::Body`
- Enum variants are automatically converted to snake_case [#12](https://github.com/TedDriggs/darling/pull/12)