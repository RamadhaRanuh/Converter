"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ImageRouter = void 0;
const express_1 = __importDefault(require("express"));
const imageController_1 = require("../controllers/imageController");
const upload_1 = __importDefault(require("../middleware/upload"));
const router = express_1.default.Router(); // Create a new router
exports.ImageRouter = router;
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
router.post('/upload', upload_1.default.single('image'), imageController_1.ImageController.uploadImage);
router.post('/convert', imageController_1.ImageController.convertImage);
router.post('/extract-layers', imageController_1.ImageController.extractPSDLayers);
