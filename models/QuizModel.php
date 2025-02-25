<?php

require_once __DIR__ . '/sheet_config.php';

class QuizModel {
    public function __construct() {
    }

    private function getDataSheet($sheetName){
        $context = stream_context_create([
            'ssl' => [
                'verify_peer' => false,
                'verify_peer_name' => false,
            ],
        ]);
        global $sheet_config;
        $csvData = file_get_contents($sheetName, false, $context);
        $data = array_map('str_getcsv', explode("\n", trim($csvData)));
        return $data;
    }

    public function getQuestionRoundList($round){
        global $sheet_config;
        $data = $this->getDataSheet($sheet_config['questions']);

        $filteredRows = [];

        for ($i = ($round - 1) * 10 + 1;  $i < ($round) * 10 + 1; $i++){
            $filteredRows[] = $data[$i];
        }
        return $filteredRows;
    }

    public function getStatisticsRoundList($round){
        global $sheet_config;
        return $this->getDataSheet($sheet_config['statistics']);
    }

    public function getLeaderBoard(){
        global $sheet_config;
        $data = $this->getDataSheet($sheet_config['leaderboard']);

        // Remove the first row
        array_shift($data);

        // Remove the first column from each row
        $data = array_map(fn($row) => array_slice($row, 1), $data);

        return $data;
    }
}