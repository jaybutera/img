<script>
    import Nav from "../components/Nav.svelte";
    export let data;
    let indexes = data.indexes;
</script>

<Nav />

<h1>Indexes</h1>
<ul>
    {#each indexes as index}
        <li><a href="/index/{index}">{index}</a></li>
    {/each}
</ul>
