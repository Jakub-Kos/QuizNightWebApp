<?php

class QuestionsViewer {
    function render($questions){
        if ($questions === null) {
            http_response_code(404);
            echo "";
            return;
        }
    
        echo '
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Presentation</title>

            <link rel="stylesheet" href="../reveal.js/dist/reveal.css">
            <link rel="stylesheet" href="../styles/black.css">
            <link rel="stylesheet" type="text/css" href="./styles/questions.css">
        </head>
        <body>
            <div class="reveal">
                <div class="slides">
        ';
        $roundName = htmlspecialchars($questions[0][0], ENT_QUOTES, 'UTF-8');
        $parts = explode("-",$roundName);
        echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>$parts[0]</h1>
                        <h1 class='r-fit-text' style='padding: 20% 0; color: #dda434'>$parts[1]</h1>
                    </section>
        ";

        for ($id = 0; $id < 10; $id++){
            $currentRow = $questions[$id];
            $number = $currentRow[1];
            $question = $currentRow[2];
            $type = $currentRow[3];
            $answer = $currentRow[4];
            if ($answer === "") $answer = "Text";

            if ($type === "Written" || $type === "Numeric"){
                echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
                        <div class='type'>$answer</div>
                    </section>
                ";
            }
            else if ($type === "ABCD"){
                echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <div class='grid-container'>
                            <div class='box'>
                                <div class='label'>A</div>
                                <div class='content'>$currentRow[4]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>B</div>
                                <div class='content'>$currentRow[5]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>C</div>
                                <div class='content'>$currentRow[6]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>D</div>
                                <div class='content'>$currentRow[7]</div>
                            </div>
                        </div>
                    </section>
                ";
            }
            else if ($type === "Yes/No"){
                echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <div class='grid-container'>
                            <div class='box'>
                                <div class='label'>Y</div>
                                <div class='content'>Yes</div>
                            </div>
                            <div class='box'>
                                <div class='label'>N</div>
                                <div class='content'>No</div>
                            </div>
                        </div>
                    </section>
                ";
            }
            else if ($type === "Image"){
                echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <img src='../source/$currentRow[9]'>
                    </section>
                ";
            }
            else if ($type === "Audio"){
                echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <audio controls>
                            <source src='../source/$currentRow[9]'>
                        </audio>
                    </section>
                ";
            }
            else if ($type === "Video"){
                echo "
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <video controls>
                            <source src='../source/$currentRow[9]'>
                        </video>
                    </section>
                ";
            }
        }
        echo '
                    <section data-background-gradient="linear-gradient(to bottom, #042335, #053049)" data-vertical-align="top">
                        <h1 style="padding: 25% 0">
                        <a href="./index.php?page=menu" class="play-button">Back to Menu</a></h1>
                    </section>
        ';

        echo '
                </div>
            </div>
            <script src="../reveal.js/dist/reveal.js"></script>
            <script src="../reveal.js/plugin/zoom/zoom.js"></script>
            <script src="../reveal.js/plugin/notes/notes.js"></script>
            <script src="../reveal.js/plugin/search/search.js"></script>
            <script src="../reveal.js/plugin/markdown/markdown.js"></script>
            <script src="../reveal.js/plugin/highlight/highlight.js"></script>
            <script>

                // Also available as an ES module, see:
                // https://revealjs.com/initialization/
                Reveal.initialize({
                    controls: true,
                    progress: true,
                    center: false,
                    hash: true,
                    width: 1920,
                    height: 1080,

                    autoSlide: 60000,
                    // Learn about plugins: https://revealjs.com/plugins/
                    plugins: [ RevealZoom, RevealNotes, RevealSearch, RevealMarkdown, RevealHighlight ]
                });

            </script>
        </body>
        </html>
        ';
    }
}