<script>
    export let data;
    const imgs = data.images;
    const img_server = "https://img.smdhi.xyz";
</script>

<style>
    body {
        margin: 0;
        padding: 0;
    }
    .grid {
        display: flex;
        justify-content: space-evenly;
        flex-wrap: wrap;
        align-items: center;
    }
    .media-container {
        max-width: 33%;
    }
    .media {
        max-width: 100%;
    }
</style>

<div class="grid">
    {#each imgs as name (name)}
        <div class="media-container">
            <a href="{ img_server }/img/{ name }">
                <img class="media" src="{img_server}/img/{ name }"/>
                <!--<img src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 3 2'%3E%3C/svg%3E" class="media lazy" data-src="{{img_server}}/img/{{ name }}"/>-->
            </a>
        </div>
    {/each}
</div>
