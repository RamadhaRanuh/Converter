import React, { useState } from 'react';

interface ConversionOptionsProps {
    file: any;
    onConvert: (data: any) => void;
}

const ConversionOptions: React.FC<ConversionOptionsProps> = ( { file, onConvert }) => {
    const [targetFormat, setTargetFormat] = useState('png');
    const [quality, setQuality] = useState(100);
    const [extractLayers, setExtractLayers] = useState(false);

    const isPSD = file?.mimetype === 'image/vnd.adobe.photoshop';

    const handleConvert = () => {
        onConvert({
            filePath: file.path,
            targetFormat,
            quality,
            extractLayers: isPSD && extractLayers
        });
    };

    return (
        <div className="conversion-options">
            <h3>Conversion Options</h3>
            <div className="file-info">
                <p><strong>File:</strong>{file.originalname}</p>
                <p><strong>Size:</strong>{(file.size / 1024 / 1024).toFixed(2)}</p>
                <p><strong>Type:</strong>{file.mimetype}</p>
            </div>

            <div className="option-form">
                <div className="form-group">
                    <label htmlFor="format">Target Format:</label>
                    <select
                        id="format"
                        value={targetFormat}
                        onChange={(e) => {setTargetFormat(e.target.value)}}
                    >
                        <option value="jpg">JPG</option>
                        <option value="png">PNG</option>
                        <option value="webp">WEBP</option>
                        <option value="tiff">TIFF</option>
                        {file.mimetype === 'image/svg+xml' && <option value="svg">SVG</option>}
                    </select>
                </div>

                <div className="form-group">
                    <label htmlFor="quality">Quality:</label>
                    <input
                        id="quality"
                        type="range"
                        min="1"
                        max="100"
                        value={quality}
                        onChange={(e) => {setQuality(Number(e.target.value))}}
                    />
                    <span>{quality}%</span>
                </div>

                {isPSD && (
                    <div className="form-group">
                        <label>
                            <input
                                type="checkbox"
                                checked={extractLayers}
                                onChange={(e) => setExtractLayers(e.target.checked)}
                            />
                            Extract Layers
                        </label>
                    </div>
                )}

                <button
                    className="convert-button"
                    onClick={handleConvert}
                >
                    Convert
                </button>
            </div>
        </div>
    );
};

export default ConversionOptions;