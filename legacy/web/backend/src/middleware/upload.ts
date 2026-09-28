import multer from 'multer';
import path from 'path';

// Configure Storage
const storage = multer.diskStorage({
    destination: (req, file, cb) => {
        cb(null, 'uploads/');
    },
    filename: (req, file, cb) => {
        cb(null, `${Date.now()}-${file.originalname}`);
    }
});

// File Filter
const fileFilter = (req: any, file: any, cb: any) => {
    const allowedMimeTypes = [
        'image/jpeg', 'image/png', 'image/bmp', 'image/webp', 'image/tiff',
        'image/svg+xml', 'application/postscript', 'image/vnd.adobe.photoshop'
    ];

    const allowedExtensions = ['.jpg', '.jpeg', '.png', '.bmp', '.webp', '.tiff', '.svg', '.psd'];
    const ext = path.extname(file.originalname).toLowerCase();
    console.log(file.mimetype);
    console.log(ext);
    if (allowedMimeTypes.includes(file.mimetype) || allowedExtensions.includes(ext))
    {
        cb(null, true);
    } 
    
    else 
    {
        cb(new Error('Unsupported file type'), false);
    }
};

const upload = multer({
    storage,
    fileFilter,
    limits: { fileSize: 10 * 1024 * 1024 } // 10 MB
});

export default upload;