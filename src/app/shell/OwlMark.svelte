<script lang="ts">
  // The Owlerlay mark. Awake = amber eyes (brand/live); asleep = dim (resting).
  // Big bold eyes so it reads as an owl even at ~24px.
  let { size = 26, awake = true }: { size?: number; awake?: boolean } =
    $props();
  const ring = $derived(awake ? "#ffb23e" : "#463f5c");
  const pupil = $derived(awake ? "#ffd089" : "#6f6885");
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 32 32"
  fill="none"
  aria-hidden="true"
>
  <!-- ear tufts -->
  <path
    d="M7.5 5.5 L11.5 10.5 M24.5 5.5 L20.5 10.5"
    stroke="#c9743e"
    stroke-width="2.6"
    stroke-linecap="round"
  />
  <!-- head -->
  <path
    d="M16 5 C8.8 5 4.3 10 4.3 17.4 C4.3 25 9.4 28.7 16 28.7 C22.6 28.7 27.7 25 27.7 17.4 C27.7 10 23.2 5 16 5 Z"
    fill="#261f37"
    stroke="#3c3357"
    stroke-width="1.4"
  />
  <!-- eyes (big, bold rings) -->
  <circle
    cx="10.7"
    cy="16.6"
    r="5.7"
    fill="#15121e"
    stroke={ring}
    stroke-width="2.1"
  />
  <circle
    cx="21.3"
    cy="16.6"
    r="5.7"
    fill="#15121e"
    stroke={ring}
    stroke-width="2.1"
  />
  <circle cx="10.7" cy="16.6" r="2.5" fill={pupil} />
  <circle cx="21.3" cy="16.6" r="2.5" fill={pupil} />
  <!-- beak -->
  <path d="M16 19.2 l-2.1 3 h4.2 Z" fill="#c9743e" />
</svg>
