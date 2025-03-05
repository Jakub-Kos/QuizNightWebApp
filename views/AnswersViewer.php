<?php

class AnswersViewer {
    function render($questions, $stats){
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
                        <h4 class='r-fit-text' style='padding: 10% 0; color: #dda434'>$parts[1]</h1>
                    </section>
        ";

        for ($id = 0; $id < 10; $id++){
            $currentRow = $questions[$id];
            $number = $currentRow[1];
            $question = $currentRow[2];
            $type = $currentRow[3];
            $answer = $currentRow[4];
            if ($answer === "") $answer = "Text";

            $correct = $currentRow[8];

            if ($type === "Written" || $type === "Numeric"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
                        <div class='type'>$answer</div>
                    </section>

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
                        <div class='type correct'>$correct</div>
                    </section>
                </section>
                ";
            }
            else if ($type === "ABCD"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
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

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
                        <div class='grid-container'>
                            <div class='box " . ($correct === "A" ? "correct" : "") . "'>
                                <div class='label'>A</div>
                                <div class='content'>$currentRow[4]</div>
                            </div>
                            <div class='box " . ($correct === "B" ? "correct" : "") . "'>
                                <div class='label'>B</div>
                                <div class='content'>$currentRow[5]</div>
                            </div>
                            <div class='box " . ($correct === "C" ? "correct" : "") . "'>
                                <div class='label'>C</div>
                                <div class='content'>$currentRow[6]</div>
                            </div>
                            <div class='box " . ($correct === "D" ? "correct" : "") . "'>
                                <div class='label'>D</div>
                                <div class='content'>$currentRow[7]</div>
                            </div>
                        </div>
                    </section>
                </section>
                ";
            }
            else if ($type === "Yes/No"){
                echo "
                <section>
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

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <div class='grid-container'>
                            <div class='box " . ($correct === "Yes" ? "correct" : "") . "'>
                                <div class='label'>Y</div>
                                <div class='content'>Yes</div>
                            </div>
                            <div class='box " . ($correct === "No" ? "correct" : "") . "'>
                                <div class='label'>N</div>
                                <div class='content'>No</div>
                            </div>
                        </div>
                    </section>
                </section>
                ";
            }
            else if ($type === "Image"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
                        <img src='../source/$currentRow[9]'>
                    </section>
                    
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h2>$question</h2>
                        <img src='../source/$currentRow[9]'>
                        <div class='type correct'>$correct</div>
                    </section>
                </section>
                ";
            }
            else if ($type === "Audio"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <audio controls>
                            <source src='../source/$currentRow[9]'>
                        </audio>
                    </section>

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <audio controls>
                            <source src='../source/$currentRow[9]'>
                        </audio>
                        <div class='type correct'>$correct</div>
                    </section>
                </section>
                ";
            }
            else if ($type === "Video"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <video controls preload='auto'>
                            <source src='../source/$currentRow[9]'>
                        </video>
                    </section>

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <video controls>
                            <source src='../source/$currentRow[9]'>
                        </video>
                        <div class='type correct'>$correct</div>
                    </section>
                </section>
                ";
            }
            else if ($type === "PImage"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <img src='../source/$currentRow[9]'>
                    </section>
                    
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <img src='../source/$currentRow[9]'>
                        <div class='type correct'>$correct</div>
                    </section>
                </section>
                ";
            }
            else if ($type === "PVideo"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <video controls>
                            <source src='../source/$currentRow[9]'>
                        </video>
                    </section>

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <video controls>
                            <source src='../source/$currentRow[9]'>
                        </video>
                        <div class='type correct'>$correct</div>
                    </section>
                </section>
                ";
            }
            else if ($type === "Sort"){
                echo "
                <section>
                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <div class='grid-container'>
                            <div class='box'>
                                <div class='label'>1</div>
                                <div class='content'>$currentRow[4]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>2</div>
                                <div class='content'>$currentRow[5]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>3</div>
                                <div class='content'>$currentRow[6]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>4</div>
                                <div class='content'>$currentRow[7]</div>
                            </div>
                        </div>
                    </section>

                    <section data-background-gradient='linear-gradient(to bottom, #042335, #053049)' data-vertical-align='top'>
                        <h1>Otázka #$number</h1>
                        <hr>
                        <h1>$question</h1>
                        <div class='grid-container'>
                            <div class='grid-container'>
                            <div class='box'>
                                <div class='label'>1</div>
                                <div class='content'>$currentRow[4]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>2</div>
                                <div class='content'>$currentRow[5]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>3</div>
                                <div class='content'>$currentRow[6]</div>
                            </div>
                            <div class='box'>
                                <div class='label'>4</div>
                                <div class='content'>$currentRow[7]</div>
                            </div>
                        </div>
                        <div class='type correct' style:'width:100%'>$correct</div>
                    </section>
                </section>
                ";
            }
        }

        echo '
                    <section data-background-gradient="linear-gradient(to bottom, #042335, #053049)" data-vertical-align="top">
                        <h1 style="padding: 25% 0">
                        <a href="./index.php?page=menu" class="play-button">Zpět</a></h1>
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