import spawn from "cross-spawn";
import path from "path";
import dotenv from "dotenv";

dotenv.config({
    path: path.resolve("../.env"),
});

if (
    !process.env.LOCAL_CDN_MEDIA_DIR ||
    process.env.LOCAL_CDN_MEDIA_DIR === ""
) {
    console.log("Media server sync: set [LOCAL_CDN_MEDIA_DIR] in .env");
    process.exit(1);
}

if (
    !process.env.REMOTE_CDN_MEDIA_BUCKET ||
    process.env.REMOTE_CDN_MEDIA_BUCKET === ""
) {
    console.log("Media server sync: set [REMOTE_CDN_MEDIA_BUCKET] in .env");
    process.exit(1);
}

const cmd = process.argv[2];

const CloudStorageMedia = `gs://${process.env.REMOTE_CDN_MEDIA_BUCKET}/`;

const localPath = path.resolve(process.env.LOCAL_CDN_MEDIA_DIR);

if (cmd === "--hard") {
    console.log(`Syncing ${cmd}`);
    console.log(`From ${localPath}`);
    console.log(`To ${CloudStorageMedia}`);
    spawn.sync("gsutil", [
        "-m",
        "rsync",
        "-d",
        "-r",
        localPath,
        CloudStorageMedia,
    ]);
} else {
    console.log("invalid option!");
}
