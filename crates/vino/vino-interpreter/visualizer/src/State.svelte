<script>
  export let schematics = [];
  export let highlight = {};

  function hightlightComponent(s) {
    return (
      (highlight.type === "port_data" &&
        s.component_index == highlight.component_index) ||
      (highlight.type === "port_status_change" &&
        s.component_index == highlight.component_index) ||
      (highlight.type === "call_complete" &&
        highlight.index == s.component_index)
    );
  }
  function hightlightPort(s) {
    return (
      (highlight.type === "port_data" &&
        s.direction == highlight.dir &&
        s.port_index == highlight.port_index &&
        s.component_index == highlight.component_index) ||
      (highlight.type === "port_status_change" &&
        s.direction == highlight.dir &&
        s.port_index == highlight.port_index &&
        s.component_index == highlight.component_index)
    );
  }

  function getNumPending(tx_id, index) {
    let schematic = schematics.find((s) => s.tx === tx_id);
    if (schematic) {
      const pending = schematic.state.find(
        (s) => s.type === "pending" && s.component_index === index
      );
      return pending ? pending.num : 0;
    } else {
      return 0;
    }
  }

  function decodeMessage(payload) {
    console.log(payload);
    return payload[0]
      ? `Success(${
          payload[0][0]
            ? MessagePack.decode(payload[0][0])
            : payload[0][2] || payload[0][1]
        })`
      : payload[3]
      ? `Signal::${payload[3]}`
      : `Failure("${payload[1][1] || payload[1][2]}")`;
  }
</script>

<main>
  {#each schematics as schematic}
    <div>
      <h3>Schematic: {schematic.schematic} (tx: {schematic.tx})</h3>
      <span>Components</span>
      <ul>
        {#each schematic.state
          .filter((s) => s.type === "component")
          .sort( (a, b) => (a.component_index < b.component_index ? -1 : a.component_index > b.component_index ? 1 : 0) ) as s}
          <li class="component {hightlightComponent(s) ? 'highlight' : ''}">
            {s.component_index}
            {s.name}
            {#if getNumPending(schematic.tx, s.component_index) > 0}
              Pending: {getNumPending(schematic.tx, s.component_index)}
            {/if}
          </li>
        {/each}
      </ul>
      <span>Ports</span>
      <ul>
        {#each schematic.state
          .filter((s) => s.type === "port")
          .sort( (a, b) => (a.component_index < b.component_index ? -1 : a.component_index > b.component_index ? 1 : 0) ) as s}
          <li
            class="port port-status status-{s.status} {hightlightPort(s)
              ? 'highlight'
              : ''}"
          >
            ({s.component_index}:{s.port_index}) {s.component}.{s.direction}.{s.port}
            {s.status}
            {#if s.pending > 0}
              <span> [Buffered {s.pending}]</span>
              <ul>
                {#each s.packets as packet}
                  <li>{decodeMessage(packet.payload)}</li>
                {/each}
              </ul>
            {/if}
          </li>
        {/each}
      </ul>
      <span>Transaction output</span>
      <ul>
        {#each schematic.state.filter((s) => s.type === "tx_output") as s}
          <li class="port {hightlightPort(s) ? 'highlight' : ''}">
            {s.component}.{s.direction}.{s.port}
            {s.status}
            {#if s.pending > 0}
              <span> [Buffered {s.pending}]</span>
            {/if}
          </li>
        {/each}
      </ul>
      <span>Connections</span>
      <ul>
        {#each schematic.state.filter((s) => s.type === "connection") as s}
          <li>{s.connection}</li>
        {/each}
      </ul>
    </div>
  {/each}
</main>

<style>
  .port-status::before {
    display: inline-block;
    width: 1em;
    line-height: 1em;
    border-radius: 20px;
    margin-right: 1em;
    text-align: center;
    color: transparent;
    content: "⚫";
  }
  .port.status-Open::before {
    text-shadow: 0 0 0 green;
  }
  .port.status-DoneYield::before {
    text-shadow: 0 0 0 rgb(15, 93, 107);
  }
  .port.status-DoneOpen::before {
    text-shadow: 0 0 0 rgb(216, 219, 16);
  }
  .port.status-DoneClosing::before {
    text-shadow: 0 0 0 rgb(255, 106, 6);
  }
  .port.status-DoneClosed::before {
    text-shadow: 0 0 0 rgb(88, 61, 61);
  }

  .port.highlight {
    background-color: rgb(151, 218, 212);
  }

  .component.highlight {
    background-color: rgb(151, 218, 212);
  }
</style>
