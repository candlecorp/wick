use wick_packet::Entity;

pub(crate) fn path_to_entity(path: &str) -> Result<Entity, &str> {
  Ok(
    path
      .split_once("::")
      .map_or_else(|| Entity::local(path), |(path, op)| Entity::operation(path, op)),
  )
}
