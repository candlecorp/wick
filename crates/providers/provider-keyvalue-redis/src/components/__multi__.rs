pub use vino_interface_keyvalue::__multi__::*;

pub(crate) async fn job(inputs: Vec<ComponentInputs>, output: OutputPorts, context: crate::Context) -> JobResult {
  let mut pipe = redis::pipe();
  let mut transaction = pipe.atomic();
  trace!("REDIS:PIPELINE:{:?}", inputs);

  let span = trace_span!("redis pipeline");
  let _guard = span.enter();

  for input in inputs {
    trace!(?input, "multi-input");
    transaction = match input {
      ComponentInputs::Delete(v) => transaction.cmd("DEL").arg(v.keys),
      ComponentInputs::Exists(v) => transaction.cmd("EXISTS").arg(v.key),
      ComponentInputs::Incr(v) => {
        if v.amount == 1 {
          transaction.cmd("INCR").arg(v.key)
        } else {
          transaction.cmd("INCRBY").arg(v.key).arg(v.amount)
        }
      }
      ComponentInputs::Decr(v) => {
        if v.amount == 1 {
          transaction.cmd("DECR").arg(v.key)
        } else {
          transaction.cmd("DECRBY").arg(v.key).arg(v.amount)
        }
      }
      ComponentInputs::KeyGet(v) => transaction.cmd("GET").arg(v.key),
      ComponentInputs::KeySet(v) => {
        if v.expires == 0 {
          transaction.cmd("SET").arg(v.key).arg(v.value)
        } else {
          transaction.cmd("SETEX").arg(v.key).arg(v.expires).arg(v.value)
        }
      }
      ComponentInputs::ListAdd(v) => {
        transaction = transaction.cmd("RPUSH").arg(v.key);
        for value in v.values {
          transaction = transaction.arg(value);
        }
        transaction
      }
      ComponentInputs::ListRange(v) => transaction.cmd("LRANGE").arg(v.key).arg(v.start).arg(v.end),
      ComponentInputs::ListRemove(v) => transaction.cmd("LREM").arg(v.key).arg(v.num).arg(v.value),
      ComponentInputs::SetAdd(v) => {
        transaction = transaction.cmd("SADD").arg(v.key);
        for value in v.values {
          transaction = transaction.arg(value);
        }
        transaction
      }
      ComponentInputs::SetContains(v) => transaction.cmd("SISMEMBER").arg(v.key).arg(v.member),
      ComponentInputs::SetGet(v) => transaction.cmd("SMEMBERS").arg(v.key),
      ComponentInputs::SetRemove(v) => transaction.cmd("SREM").arg(v.key).arg(v.values),
      ComponentInputs::SetScan(v) => {
        let cursor_str = v.cursor;
        let cursor: u64 = cursor_str
          .parse()
          .map_err(|_| crate::Error::CursorConversion(cursor_str))?;
        transaction
          .cmd("SSCAN")
          .arg(v.key)
          .arg(cursor)
          .arg("MATCH")
          .arg("*")
          .arg("COUNT")
          .arg(v.count)
      }
    }
  }
  let result = context.run_pipeline(transaction).await?;
  trace!(?result, "pipeline result");
  output.result.done(Payload::success(&true))?;

  Ok(())
}
