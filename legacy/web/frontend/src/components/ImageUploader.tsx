import React, { useState } from 'react';
import { useDropzone } from 'react-dropzone';
import axios from 'axios';

interface ImageUploaderProps {
    onUploadSuccess: (data: any) => void;
}

const ImageUploader: React.FC<ImageUploaderProps> = ( { onUploadSuccess } ) => {
    const [isUploading, setIsUploading] = useState(false);
    const [uploadProgress, setUploadProgress] = useState(0);
    const [error, setError] = useState<String | null>(null);

    const { getRootProps, getInputProps, isDragActive } = useDropzone({
        accept: {
            'image/*': ['.jpg', '.jpeg', '.png', '.webp', '.tiff', '.svg'],
            'application/postscript': ['.ai'],
            'image/vnd.adobe.photoshop': ['.psd']
        },
        maxSize: 10485760, // 10MB
        multiple: false,

        onDrop: async (acceptedFiles, rejectedFiles) => {
            if (rejectedFiles.length > 0){
                setError('File rejected: Check file type and size (max 10MB)')
                return;
            }

            if (acceptedFiles.length === 0) return;

            setIsUploading(true);
            setError(null);
            setUploadProgress(0);

            const file = acceptedFiles[0];
            const formData = new FormData();
            formData.append('image', file);

            try {
                const response = await axios.post('http://localhost:3000/api/images/upload', formData, {
                    headers: {
                        'Content-Type': 'multipart/form-data'
                    },
                    onUploadProgress: (progressEvent) => {
                        const percentCompleted = progressEvent.total ? Math.round((progressEvent.loaded * 100) / progressEvent.total) : 0;
                        setUploadProgress(percentCompleted);
                    }
                });
                setIsUploading(false);
                onUploadSuccess(response.data);
            } catch (err: any){
                setError(err.response?.data?.error || 'Something went wrong. Please try again.');
                setIsUploading(false);
            }
        }
    });

    return (
        <div className="image-uploader">
            <div {...getRootProps()} 
            className = {`dropzone ${isDragActive ? 'active' : ''}`}
            >
                <input {...getInputProps()} />
                {isUploading ? (
                    <div className="upload-progress">
                        <div className="progress-bar">
                            <div
                                className="progress-bar-fill"
                                style={{ width: `${uploadProgress}%` }}
                            ></div>
                        </div>
                        <p>Uploading: {uploadProgress}%</p>
                    </div>
                ) : (
                    <div className="upload-prompt">
                        <p>Drag and drop an image here, or click to select a file</p>
                        <p className="supported-formats">
                            Supported formats: JPG, PNG, WEBP, BMP, TIFF, SVG, AI, PSD
                        </p>
                    </div>
                )}
            </div>
            {error && <p className="error">{error}</p>}
        </div>
    );
};

export default ImageUploader;
