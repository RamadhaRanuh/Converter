import express from 'express';
import multer from 'multer';
import path from 'path';
import { ImageController } from '../controllers/imageController';
import { Upload } from '../middleware/upload';


const router = express.Router(); // Create a new router
imageController = new ImageController();
upload = new Upload();

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

router.post('/upload', upload.single('image'), imageController.uploadImage);
router.post('/convert', imageController.convertImage);
router.post('/extract-layers', imageController.extractPSDLayers);


export { router as ImageRouter };

