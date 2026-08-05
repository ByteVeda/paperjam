import { useCallback, useState } from 'react';
import styles from './playground.module.css';

interface Props {
  onFileLoaded: (data: Uint8Array, fileName: string) => void;
}

export default function PdfUploader({ onFileLoaded }: Props) {
  const [dragOver, setDragOver] = useState(false);
  const [fileName, setFileName] = useState<string | null>(null);

  const handleFile = useCallback(
    (file: File) => {
      const reader = new FileReader();
      reader.onload = (e) => {
        const data = new Uint8Array(e.target?.result as ArrayBuffer);
        setFileName(file.name);
        onFileLoaded(data, file.name);
      };
      reader.readAsArrayBuffer(file);
    },
    [onFileLoaded],
  );

  return (
    <div
      className={`${styles.dropZone} ${dragOver ? styles.dropZoneActive : ''}`}
      onDragOver={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragLeave={() => setDragOver(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragOver(false);
        if (e.dataTransfer.files[0]) handleFile(e.dataTransfer.files[0]);
      }}
    >
      <p>
        {fileName
          ? `Loaded: ${fileName}`
          : 'Drop a PDF here or click to upload'}
      </p>
      <input
        type="file"
        accept=".pdf"
        className={styles.fileInput}
        onChange={(e) => {
          if (e.target.files?.[0]) handleFile(e.target.files[0]);
        }}
      />
    </div>
  );
}
