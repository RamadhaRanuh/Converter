import React from 'react';

interface ConversionResultProps {
    result: any;
}

const ConversionResult: React.FC<ConversionResultProps> = ( { result } ) => {
    if (!result) return null;

    const getFilename = (filepath: string) => {
        return filepath.split('/').pop() || filepath;
    };

    return (

        <div className="conversion-result">
            <h3>Conversion Result</h3>

            {result.convertedPath && (
                <div className="result-item">
                    <h4>Converted File</h4>
                    <p>Format: {result.targetFormat}</p>
                    <p>Path: {result.convertedPath}</p>
                    <a 
                        href={`http://localhost:3000/${result.convertedPath}`}
                        download={getFilename(result.convertedPath)}
                        className="download-button"
                    >
                    Download
                    </a>
                </div>
            )}

            {result.layers && (
                <div className="layers-result">
                    <h4>Extracted Layers</h4>
                    <p>Layers Directory: {result.layersDir}</p>
                    <div className="layers-list">
                        {result.layers.map((layer: any, index: any) => (
                            <div key={index} className="layer-item">
                                <p>{layer.name}</p>
                                <a
                                    href={`http://localhost:3000/${layer.path}`}
                                    download={getFilename(layer.path)}
                                    className="download-button small"
                                >
                                    Download
                                </a>
                            </div>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
};

export default ConversionResult;
