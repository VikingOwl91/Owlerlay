<script lang="ts">
  import type { CountdownState } from "../../features/countdown/model/countdown.types";

  // A status "eye": glows amber awake (Running), dim asleep (Idle),
  // cool (Paused), pale (Finished). Encodes the real state machine.
  let { state, size = 13 }: { state: CountdownState; size?: number } = $props();
</script>

<span
  class="eye"
  class:running={state === "Running"}
  class:paused={state === "Paused"}
  class:finished={state === "Finished"}
  style="--sz:{size}px"
  aria-hidden="true"
></span>

<style>
  .eye {
    display: inline-block;
    width: var(--sz);
    height: var(--sz);
    border-radius: 50%;
    background: var(--st-idle);
    box-shadow: inset 0 0 0 2px rgba(0, 0, 0, 0.25);
  }
  .running {
    background: radial-gradient(
      circle at 50% 40%,
      var(--eye-bright),
      var(--eye) 70%
    );
    box-shadow: var(--glow);
    animation: owl-pulse 2.4s ease-in-out infinite;
  }
  .paused {
    background: var(--st-paused);
    box-shadow: none;
  }
  .finished {
    background: var(--st-finished);
    box-shadow: none;
  }
</style>
