// deno-lint-ignore-file no-explicit-any
export interface CloudEvent<T> {
  /**
   * Identifies the event. Producers MUST ensure that `source` + `id`
   * is unique for each distinct event. If a duplicate event is re-sent (e.g. due
   * to a network error) it MAY have the same `id`. Consumers MAY assume that
   * Events with identical `source` and `id` are duplicates.
   */
  id: string;
  /**
   * Identifies the context in which an event happened. Often this
   * will include information such as the type of the event source, the
   * organization publishing the event or the process that produced the event. The
   * exact syntax and semantics behind the data encoded in the URI is defined by
   * the event producer.
   *
   * Producers MUST ensure that `source` + `id` is unique for each distinct event.
   *
   * An application MAY assign a unique `source` to each distinct producer, which
   * makes it easy to produce unique IDs since no other producer will have the same
   * source. The application MAY use UUIDs, URNs, DNS authorities or an
   * application-specific scheme to create unique `source` identifiers.
   *
   * A source MAY include more than one producer. In that case the producers MUST
   * collaborate to ensure that `source` + `id` is unique for each distinct event.
   */
  source: string;
  /**
   * The version of the CloudEvents specification which the event
   * uses. This enables the interpretation of the context. Compliant event
   * producers MUST use a value of `1.0` when referring to this version of the
   * specification.
   *
   * Currently, this attribute will only have the 'major' and 'minor' version
   * numbers included in it. This allows for 'patch' changes to the specification
   * to be made without changing this property's value in the serialization.
   * Note: for 'release candidate' releases a suffix might be used for testing
   * purposes.
   */
  specversion: string;
  /**
   * This attribute contains a value describing the type of event
   * related to the originating occurrence. Often this attribute is used for
   * routing, observability, policy enforcement, etc. The format of this is
   * producer defined and might include information such as the version of the
   * `type`.
   */
  type: string;
  /**
   * Content type of `data` value. This attribute enables `data` to
   * carry any type of content, whereby format and encoding might differ from that
   * of the chosen event format. For example, an event rendered using the
   * [JSON envelope](formats/json-format.md#3-envelope) format might carry an XML
   * payload in `data`, and the consumer is informed by this attribute being set to
   * "application/xml". The rules for how `data` content is rendered for different
   * `datacontenttype` values are defined in the event format specifications; for
   * example, the JSON event format defines the relationship in
   * [section 3.1](formats/json-format.md#31-handling-of-data).
   *
   * For some binary mode protocol bindings, this field is directly mapped to the
   * respective protocol's content-type metadata property. Normative rules for the
   * binary mode and the content-type metadata mapping can be found in the
   * respective protocol.
   *
   * In some event formats the `datacontenttype` attribute MAY be omitted. For
   * example, if a JSON format event has no `datacontenttype` attribute, then it is
   * implied that the `data` is a JSON value conforming to the "application/json"
   * media type. In other words: a JSON-format event with no `datacontenttype` is
   * exactly equivalent to one with `datacontenttype="application/json"`.
   *
   * When translating an event message with no `datacontenttype` attribute to a
   * different format or protocol binding, the target `datacontenttype` SHOULD be
   * set explicitly to the implied `datacontenttype` of the source.
   */
  datacontenttype: string;
  /**
   * Identifies the schema that `data` adheres to. Incompatible
   * changes to the schema SHOULD be reflected by a different URI.
   */
  dataschema: string;
  /**
   * This describes the subject of the event in the context of the
   * event producer (identified by `source`). In publish-subscribe scenarios, a
   * subscriber will typically subscribe to events emitted by a `source`, but the
   * `source` identifier alone might not be sufficient as a qualifier for any
   * specific event if the `source` context has internal sub-structure.
   *
   * Identifying the subject of the event in context metadata (opposed to only in
   * the `data` payload) is particularly helpful in generic subscription filtering
   * scenarios where middleware is unable to interpret the `data` content. In the
   * above example, the subscriber might only be interested in blobs with names
   * ending with '.jpg' or '.jpeg' and the `subject` attribute allows for
   * constructing a simple and efficient string-suffix filter for that subset of
   * events.
   */
  subject: string;
  /**
   * Timestamp of when the occurrence happened. If the time of the
   * occurrence cannot be determined then this attribute MAY be set to some other
   * time (such as the current time) by the CloudEvents producer, however all
   * producers for the same `source` MUST be consistent in this respect. In other
   * words, either they all use the actual time of the occurrence or they all use
   * the same algorithm to determine the value used.
   */
  time: Date;
  /**
   * Domain-specific information about the occurrence (i.e. the payload). This might
   * include information about the occurrence, details about the data that was
   * changed, or more. See the [Event Data](#event-data) section for more
   * information.
   */
  data: T;
  /**
   * CloudEvent Attributes Extensions
   */
  [attribute: string]: any;
}
