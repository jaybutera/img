<script>

    import 'bootstrap/dist/css/bootstrap.min.css';
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
    .new-topic {
        width: 50%;
        height: 50px;
        font-size: 20px;
        border-radius: 10px;
        border: 1px solid #ccc;
        padding: 10px;
        margin: 30px;
    }
    .input-images {
        width: 50%;
        height: 50px;
        font-size: 20px;
        border-radius: 10px;
        border: 1px solid #ccc;
        padding: 10px;
        margin: 10px;
    }
    .submit-topic {
        width: 50%;
        height: 50px;
        font-size: 20px;
        border-radius: 10px;
        border: 1px solid #ccc;
        padding: 10px;
        margin: 30px;
        background-color: #ccc;
    }
    h1 {
        margin-top: 100px;
        text-align: center;
    }
</style>

<h1>Start a New Collection!</h1>
<div class="new-form">
    <input type="text" class="new-topic" placeholder="topic name">
    <input type="file" multiple class="input-images">
    <button class="submit-topic">Submit</button>
</div>

