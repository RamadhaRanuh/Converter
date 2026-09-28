import { useState } from 'react';
import axios from 'axios';
import './App.css';
import ImageUploader from './components/ImageUploader';
import ConversionOptions from './components/ConversionOptions';
import ConversionResult from './components/ConversionResult';


function App() {
  const [uploadedFile, setUploadedFile] = useState<any>(null);
  const [conversionResult, setConversionResult] = useState<any>(null);
  const [isConverting, setIsConverting] = useState(false);
  const [error, setError] = useState<String | null>(null);


  const handleUploadSuccess = (data: any) => {
    setUploadedFile(data.file);
    setConversionResult(null);
    setError(null);
  };

  const handleConvert = async (options: any) => {
    setIsConverting(true);
    setError(null);

    try {
      let response: any;

      if (options.extractLayers){
        response = await axios.post('http://localhost:3000/api/images/extract-layers', {
          filePath: options.filePath,
          targetFormat: options.targetFormat
        });
      } else {
        response = await axios.post('http://localhost:3000/api/images/convert', {
          filePath: options.filePath,
          targetFormat: options.targetFormat,
          quality: options.quality
        });
      }

      setConversionResult(response.data);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Conversion failed')
    } finally {
      setIsConverting(false);
    }
  };

return (
  <div className="app">
    <header>
      <h1>Image Converter</h1>
      <p>High-performance image format conversion tool</p>
    </header>
    
    <main>
      {!uploadedFile ? (
        <ImageUploader onUploadSuccess={handleUploadSuccess} />
      ) : (
        <div className="conversion-container">
          <ConversionOptions
            file={uploadedFile}
            onConvert={handleConvert}
          />
          
          {isConverting && (
            <div className="converting-indicator">
              <div className="spinner"></div>
              <p>Converting...</p>
            </div>
          )}
          
          {error && <p className="error-message">{error}</p>}
          
          <ConversionResult result={conversionResult} />
          
          <button
            className="reset-button"
            onClick={() => {
              setUploadedFile(null);
              setConversionResult(null);
            }}
          >
            Convert Another Image
          </button>
        </div>
      )}
    </main>
  </div>
  );
}

export default App;
