<?php
require_once __DIR__ . '/../views/MenuViewer.php';

class MenuController{
    private $view;
    public function __construct() {
        $this->view = new MenuViewer();
    }

    public function handle() {
        $this->view->render();
    }
}