import { useState, useEffect } from "react";
import Papa from "papaparse";

export function useQuizData(url) {
  const [data, setData] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Papa.parse(url, {
      download: true,
      header: false, // Your CSVs seem to lack headers in some parts
      complete: (results) => {
        setData(results.data);
        setLoading(false);
      },
    });
  }, [url]);

  return { data, loading };
}
