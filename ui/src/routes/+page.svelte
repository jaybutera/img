<script>
    import 'bootstrap/dist/css/bootstrap.min.css';
    import Nav from '../components/Nav.svelte';
    export let data;
    const imgs = data.images;
</script>

<style>
    body {
        height: 100%;
    }
    .new-form {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-top: 100px;
    }
    .nt-form {
        width: 30%;
        height: 50px;
        font-size: 20px;
        border-radius: 10px;
        border: 1px solid #ccc;
        padding: 10px;
    }
    .new-topic {
        margin: 30px;
    }
    .input-images {
        margin: 10px;
    }
    .submit-topic {
        margin: 30px;
        background-color: #ccc;
    }
    h1 {
        margin-top: 100px;
        text-align: center;
    }
</style>

<Nav />
<h1>Start a New Collection!</h1>
<div class="new-form">
        <input type="text" class="nt-form new-topic" placeholder="topic name">
        <input type="file" multiple class="nt-form input-images">
        <button class="nt-form submit-topic">Submit</button>
</div>
