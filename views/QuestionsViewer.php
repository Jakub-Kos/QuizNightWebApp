<?php

class QuestionsViewer {
    function render($questions){
        if ($questions === null) {
            http_response_code(404);
            echo "";
            return;
        }
    
        echo "
        <!DOCTYPE html>
        <html lang='en'>
        <head>
            <meta charset='UTF-8'>
            <title>Questions</title>
            <link rel='stylesheet' type='text/css' href='./styles/main.css'>
        </head>
        <body>
            <h1>Article list</h1>
            <hr>
            <table>
                <thead>
                    <tr>
                        <th>Filter favourites?</th>
                        <th>
                            <input type='checkbox' class='filter' id='filter'>
                        </th>
                    </tr>
                </thead>
                <tbody id='articles'>
        ";
        echo "
            $questions
        ";
        echo "
                </tbody>
            </table>
            <hr>
            <div class='buttons'>
                <button id='previous'>Previous</button>
                <button id='next'>Next</button>
                <span id='page'>Page 1 / 3</span>
                <button id='createArticle'>Create article</button>
            </div>

            <dialog id='createArticlePopup' style='display: none;'>
                <form action='' id='createArticleForm' method='POST'>
                    <h2>Create Article</h2>
                    <label>Name
                        <input type='text' id='articleName' name='name' maxlength='32' placeholder='Enter article name' required>
                    </label>    
                    <div class='buttons'>
                        <button type='submit' id='createButton'>Create</button>
                        <button type='button' id='cancelButton'>Cancel</button>
                    </div>
                </form>
            </dialog>

            <script src='./scripts/pagination.js'></script>
            <script src='./scripts/popup.js'></script>
        </body>
        </html>";
    }
}