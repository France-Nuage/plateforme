//! JWT Claims implementation following RFC 7519.
//!
//! This module provides data structures for parsing and working with JWT (JSON Web Token)
//! claims according to [RFC 7519]. JWT claims represent information about an entity
//! (typically, the user) and additional data that can be used by applications to make
//! authorization decisions.
//!
//! ## Claim Categories
//!
//! JWT claims are organized into three categories:
//! - **Registered Claims**: Standard claims defined by RFC 7519
//! - **Public Claims**: Claims defined in the IANA "JSON Web Token Claims" registry
//! - **Private Claims**: Custom claims agreed upon by parties using the JWT
//!
//! ## Non-standard Claims
//!
//! This implementation includes commonly-used claims that extend beyond RFC 7519:
//! - **email**: User email address (widely used by OIDC providers for user identification)
//!
//! ## Registered Claims
//!
//! This module implements the standard registered claims defined in [Section 4.1] of RFC 7519.
//! These claims provide a consistent foundation for JWT usage across different applications
//! and services.
//!
//! ## Time-based Claims
//!
//! Several claims represent time as "NumericDate" values - the number of seconds since
//! the Unix epoch (1970-01-01T00:00:00Z UTC). This format is used for:
//! - Token expiration time (`exp`)
//! - Not-before time (`nbf`)  
//! - Issued-at time (`iat`)
//!
//! [RFC 7519]: https://tools.ietf.org/html/rfc7519
//! [Section 4.1]: https://tools.ietf.org/html/rfc7519#section-4.1

use serde::Deserialize;

/// [JWT Claims](https://datatracker.ietf.org/doc/html/rfc7519#section-4)
///
/// The JWT Claims Set represents a JSON object whose members are the claims
/// conveyed by the JWT.  The Claim Names within a JWT Claims Set MUST be
/// unique; JWT parsers MUST either reject JWTs with duplicate Claim Names or
/// use a JSON parser that returns only the lexically last duplicate member
/// name, as specified in [Section 15.12](https://datatracker.ietf.org/doc/html/rfc7519#section-15.12)
/// ("The JSON Object") of ECMAScript 5.1 [ECMAScript](https://datatracker.ietf.org/doc/html/rfc7519#ref-ECMAScript).
///
/// The set of claims that a JWT must contain to be considered valid is context
/// dependent and is outside the scope of this specification. Specific
/// applications of JWTs will require implementations to understand and process
/// some claims in particular ways.  However, in the absence of such
/// requirements, all claims that are not understood by implementations MUST
/// be ignored.
///
/// There are three classes of JWT Claim Names: Registered Claim Names, Public
/// Claim Names, and Private Claim Names.

/// [Registered Claim Names](https://datatracker.ietf.org/doc/html/rfc7519#section-4.1)
///
/// The following Claim Names are registered in the IANA "JSON Web Token Claims"
/// registry established by [Section 10.1](https://datatracker.ietf.org/doc/html/rfc7519#section-10.1).
/// None of the claims defined below are intended to be mandatory to use or
/// implement in all cases, but rather they provide a starting point for a set
/// of useful, interoperable claims. Applications using JWTs should define which
/// specific claims they use and when they are required or optional. All the
/// names are short because a core goal of JWTs is for the representation to be
/// compact.
#[derive(Clone, Debug, Deserialize)]
pub struct Claim {
    /// Audience Claim.
    ///
    /// The "aud" (audience) claim identifies the recipients that the JWT is
    /// intended for. Each principal intended to process the JWT MUST identify
    /// itself with a value in the audience claim.  If the principal processing
    /// the claim does not identify itself with a value in the "aud" claim when
    /// this claim is present, then the JWT MUST be rejected. In the general
    /// case, the "aud" value is an array of case-sensitive strings,
    /// each containing a StringOrURI value. In the special case when the JWT
    /// has one audience, the "aud" value MAY be a single case-sensitive string
    /// containing a StringOrURI value. The interpretation of audience values is
    /// generally application specific. Use of this claim is OPTIONAL.
    pub aud: Option<String>,

    /// Email Claim (Non-standard but commonly used).
    ///
    /// The "email" claim provides the email address associated with the JWT subject.
    /// While not part of the RFC 7519 standard, this claim is commonly included
    /// by OIDC providers and is widely used for user identification and authorization.
    ///
    /// **Note**: This field is used for temporary database-backed user authorization
    /// and will be replaced by standard `sub` claim processing when migrating to
    /// SpiceDB for stateless authorization.
    pub email: Option<String>,

    /// Expiration Time Claim.
    ///
    /// The "exp" (expiration time) claim identifies the expiration time on or
    /// after which the JWT MUST NOT be accepted for processing. The processing
    /// of the "exp" claim requires that the current date/time MUST be before
    /// the expiration date/time listed in the "exp" claim. Implementers MAY
    /// provide for some small leeway, usually no more than a few minutes, to
    /// account for clock skew. Its value MUST be a number containing a
    /// NumericDate value. Use of this claim is OPTIONAL.
    pub exp: Option<u64>,

    /// Issued At Claim.
    ///
    /// The "iat" (issued at) claim identifies the time at which the JWT was
    /// issued. This claim can be used to determine the age of the JWT. Its
    /// value MUST be a number containing a NumericDate value. Use of this claim
    /// is OPTIONAL.
    pub iat: Option<u64>,

    /// Issuer Claim.
    ///
    /// The [iss](https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.1)
    /// (issuer) claim identifies the principal that issued the JWT.
    /// The processing of this claim is generally application specific. The
    /// "iss" value is a case-sensitive string containing a StringOrURI value.
    /// Use of this claim is OPTIONAL.
    pub iss: Option<String>,

    /// JWT ID Claim.
    ///
    /// he "jti" (JWT ID) claim provides a unique identifier for the JWT. The
    /// identifier value MUST be assigned in a manner that ensures that there is
    /// a negligible probability that the same value will be accidentally
    /// assigned to a different data object; if the application uses multiple
    /// issuers, collisions MUST be prevented among values produced by different
    /// issuers as well. The "jti" claim can be used to prevent the JWT from
    /// being replayed. The "jti" value is a case-sensitive string. Use of this
    /// claim is OPTIONAL.
    pub jti: Option<String>,

    /// Not Before Claim.
    ///
    /// The "nbf" (not before) claim identifies the time before which the JWT
    /// MUST NOT be accepted for processing. The processing of the "nbf" claim
    /// requires that the current date/time MUST be after or equal to the
    /// not-before date/time listed in the "nbf" claim.  Implementers MAY provide
    /// for some small leeway, usually no more than a few minutes, to account
    /// for clock skew. Its value MUST be a number containing a NumericDate
    /// value. Use of this claim is OPTIONAL.
    pub nbf: Option<u64>,

    /// Subject Claim.
    ///
    /// The "sub" (subject) claim identifies the principal that is the subject
    /// of the JWT. The claims in a JWT are normally statements about the
    /// subject. The subject value MUST either be scoped to be locally unique in
    /// the context of the issuer or be globally unique. The processing of this
    /// claim is generally application specific. The "sub" value is a
    /// case-sensitive string containing a StringOrURI value.  Use of this claim
    /// is OPTIONAL.
    pub sub: Option<String>,
}
