import express from 'express';
import multer from 'multer';
import path from 'path';
import { ImageController } from '../controllers/imageController';
import upload from '../middleware/upload';


const router = express.Router(); // Create a new router

// Configure Storage
/*
const storage = multer.diskStorage({
    destination: (req, file, cb) => {
        cb(null, path.join(__dirname, '../../uploads'));
    },
    filename: (req, file, cb) => {
        const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
        const ext = path.extname(file.originalname);
        cb(null, 'original-' + uniqueSuffix + ext);;
    }
});
*/

router.post('/upload', upload.single('image'), ImageController.uploadImage);
router.post('/convert', ImageController.convertImage);
router.post('/extract-layers', ImageController.extractPSDLayers);


export { router as ImageRouter };

