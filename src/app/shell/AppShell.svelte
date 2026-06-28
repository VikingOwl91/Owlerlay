<script lang="ts">
  import CountdownPage from "../../features/countdown/components/CountdownPage.svelte";

  type Page = "countdowns" | "about";
  let activePage = $state<Page>("countdowns");
</script>

<header class="container-fluid">
  <nav>
    <ul>
      <li><strong>Owlerlay</strong></li>
    </ul>
    <ul>
      <li>
        <a
          href="#countdowns"
          onclick={(e) => {
            e.preventDefault();
            activePage = "countdowns";
          }}>Countdowns</a
        >
      </li>
      <li>
        <a
          href="#about"
          onclick={(e) => {
            e.preventDefault();
            activePage = "about";
          }}>About</a
        >
      </li>
    </ul>
  </nav>
</header>

<main class="container">
  {#if activePage === "countdowns"}
    <CountdownPage />
  {:else if activePage === "about"}
    <p>Owlerlay — OBS overlay control center.</p>
  {/if}
</main>
