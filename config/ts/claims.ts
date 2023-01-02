// deno-lint-ignore-file no-explicit-any
export interface Claims {
  /** Issuer */
  iss: string;
  /** Subject */
  sub: string;
  /** Audience */
  aud: string;
  /** Expiration Time */
  exp: Date;
  /** Not before */
  nbf: Date;
  /** Issued at */
  iat: Date;
  /** JWT ID */
  jti: string;
  /** Full name */
  name: string;
  /** Given name(s) or first name(s) */
  given_name: string;
  /** Surname(s) or last name(s) */
  family_name: string;
  /** Middle name(s) */
  middle_name: string;
  /** Casual name */
  nickname: string;
  /** Shorthand name by which the End-User wishes to be referred to */
  preferred_username: string;
  /** Profile page URL */
  profile: string;
  /** Profile picture URL */
  picture: string;
  /** Web page or blog URL */
  website: string;
  /** Preferred e-mail address */
  email: string;
  /** True if the e-mail address has been verified; otherwise false */
  email_verified: string;
  /** Gender */
  gender: string;
  /** Birthday */
  birthdate: Date;
  /** Time zone */
  zoneinfo: string;
  /** Locale */
  locale: string;
  /** Preferred telephone number */
  phone_number: string;
  /** True if the phone number has been verified; otherwise false */
  phone_number_verified: string;
  /** Preferred postal address */
  address: string;
  /** Time the information was last updated */
  updated_at: Date;
  /** Extensions */
  [claim: string]: any;
}
