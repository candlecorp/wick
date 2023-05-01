use wick_packet::Entity;

pub(crate) fn path_to_entity(path: &str) -> Result<Entity, &str> {
  let (path, op) = path.split_once("::").ok_or(path)?;
  Ok(Entity::operation(path, op))
}
