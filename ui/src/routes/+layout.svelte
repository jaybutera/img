<style>
    nav {
        display: flex;
        justify-content: space-around;
        padding: 1em;
        background-color: #f5f5f5;
        border: 1px solid #ddd;
    }
    body {
        margin: 0;
        font-family: Arial, Helvetica, sans-serif;
        background-color: black;
    }
    .topnav {
        display: flex;
        overflow: hidden;
        flex-wrap: wrap;
        background-color: #333;
    }
    .topnav a {
        float: left;
        color: #f2f2f2;
        text-align: center;
        padding: 14px 16px;
        text-decoration: none;
        font-size: 17px;
    }
    #nav-title p {
        color: #f2f2f2;
        text-align: center;
        margin: 0px;
        padding: 0px 16px;
        text-decoration: none;
        font-size: 32px;
    }
    #nav-title {
        display: flex;
        flex-direction: column;
        justify-content: center;
    }
    #nav-btns-container {
        display: flex;
        flex-grow: 1;
        flex-direction: row-reverse;
    }
    .topnav a:hover {
        background-color: #ddd;
        color: black;
    }

</style>

<slot />
