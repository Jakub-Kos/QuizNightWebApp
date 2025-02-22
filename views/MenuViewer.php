<?php

class MenuViewer {
    function render(){
        echo file_get_contents(__DIR__ . '/menu.html');
    }
}