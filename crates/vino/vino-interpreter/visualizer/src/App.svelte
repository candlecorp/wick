<script>
  import { onMount } from "svelte";
  import { crossfade } from "svelte/transition";

  export let events = [];
  import CallComplete from "./events/CallComplete.svelte";
  import PortData from "./events/PortData.svelte";
  import TransactionStart from "./events/TransactionStart.svelte";
  import PortStatusChange from "./events/PortStatusChange.svelte";
  import Invocation from "./events/Invocation.svelte";
  import TransactionDone from "./events/TransactionDone.svelte";
  import State from "./State.svelte";

  let index = parseInt(location.hash.substring(1), 10) || 0;

  const BlockTypes = {
    port_data: PortData,
    tx_start: TransactionStart,
    call_complete: CallComplete,
    tx_done: TransactionDone,
    port_status_change: PortStatusChange,
    invocation: Invocation,
  };

  async function fetchLog(evt) {
    evt.preventDefault();
    render();
    return false;
  }

  async function render() {
    const log = document.getElementById("event_log").value;
    let response = await fetch(log);
    let json = await response.json();
    console.log(json);
    events = json;
  }

  function next() {
    index = Math.min(events.length - 1, index + 1);

    location.hash = index;
  }

  function prev() {
    index = Math.max(0, index - 1);
    location.hash = index;
  }

  function go(i) {
    index = i;
    location.hash = index;
  }

  document.addEventListener("keydown", (evt) => {
    console.log(evt.code);
    if (evt.key == "ArrowRight") {
      next();
    } else if (evt.key == "ArrowLeft") {
      prev();
    }
  });

  onMount(render);
</script>

<main>
  <form on:submit={fetchLog}>
    <input type="text" id="event_log" value="/event_loop.json" />
    <button>Fetch</button>
  </form>

  <ol class="event-list">
    {#each events as event, i}
      <li class="nav {index == i ? 'highlight' : ''} type-{event.event.type}">
        <button on:click={() => go(i)}>{i}</button>
      </li>
    {/each}
  </ol>
  <button on:click={prev}>Prev</button>
  <button on:click={next}>Next</button>

  {#if events[index]}
    <div>
      <h2 transition:crossfade>{index}: {events[index].event.type}</h2>
    </div>
    <div>
      <svelte:component
        this={BlockTypes[events[index].event.type]}
        {...events[index]}
      />
    </div>
    <div class="state">
      <State
        schematics={events[index].state || []}
        highlight={events[index].event}
      />
    </div>
  {/if}
</main>

<style>
  main {
    padding: 1em;
    margin: 0 auto;
  }
  .event-list {
    list-style: none;
  }
  .event-list li {
    margin: 0px;
    padding: 0px;
    display: inline;
  }
  .event-list li.type-port_data button {
    color: blue !important;
  }
  .event-list li.type-call_complete button {
    color: rgb(71, 71, 0) !important;
  }
  .event-list li.type-tx_done button {
    color: red !important;
  }
  .event-list li.type-tx_start button {
    color: green !important;
  }
  .event-list button:hover {
    background-color: azure;
    font-weight: bold;
  }
  .nav.highlight button {
    background-color: yellow;
    font-weight: bold;
    padding: 2px;
  }

  .event-list button {
    padding: 0px;
    border: 0;
    background-color: inherit;
    text-decoration: underline;
    cursor: pointer;
  }
</style>
