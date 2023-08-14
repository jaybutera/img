<script>
    import Nav from "../../../components/Nav.svelte";
    import { img_server, handle_file_upload } from "$lib/img.ts";
    import { goto } from "$app/navigation";
    import Uploading from '../../../components/Uploading.svelte';
    export let data;
    const index = data.index_name;
    const topics = data.topics;
    const image_names = data.image_names;
</script>

<style>
    .media {
        max-width: 100%;
    }
    .center-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
    }
    .sub-container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
    }
    .container {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        max-width: 66%;
    }
    .sub-container {
    position: relative;
    display: inline-block; /* So the container will size itself to the image */
    width: 100%; /* Take full width */
    font-size: 0; /* This will prevent unwanted space between inline-block elements */
}

.media {
    display: block;  /* Remove default image margin */
    max-width: 100%; /* To ensure the image scales responsively */
    width: 100%; /* To ensure the image takes full width of its container */
    height: auto;
}

h2 {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%); /* Centers the text both vertically and horizontally */
    font-size: calc(5vw + 1vmin); /* Responsive font size using a combination of viewport width and minimum value between viewport height and width */
    font-weight: bold;
    color: white;  /* Adjust color if needed */
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.5);  /* Optional: for better text visibility against bright images */
    width: 80%; /* Adjusts width to 80% of the container */
    text-align: center; /* Center align text */
    white-space: nowrap; /* Prevents text from wrapping to the next line */
    overflow: hidden; /* Ensures text doesn't spill out even if it doesn't fit */
    text-overflow: ellipsis; /* Adds ... for overflowed content */
}
</style>

<Nav>
</Nav>

<div class="center-container">
<div class="container">
    {#each topics as topic, i}
            <a href="/{topic}">
                <div class="sub-container">
                    <h2>{topic.replaceAll('-', ' ')}</h2>
                    <img class="media" src="{img_server}/img/{image_names[i]}"/>
                </div>
            </a>
    {/each}
</div>
</div>
