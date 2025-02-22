<?php

class MainViewer {
    function render(){
        echo file_get_contents(__DIR__ . '/intro.html');
    }
}