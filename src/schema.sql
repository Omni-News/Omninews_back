
--CASCADE가 적용된 관계:
--
--    rss_item.channel_id → rss_channel.channel_id
--    embedding.channel_id → rss_channel.channel_id
--    embedding.rss_id → rss_item.rss_id
--    embedding.news_id → news.news_id
--    user_subscription_channel.user_id → user.user_id
--    user_subscription_channel.channel_id → rss_channel.channel_id
--    rss_folder.user_id → user.user_id
--    channels_in_folder.folder_id → rss_folder.folder_id
--    channels_in_folder.channel_id → rss_channel.channel_id
--    rss_css_channel.channel_id → rss_channel.channel_id

DROP TABLE IF EXISTS embedding;
DROP TABLE IF EXISTS feedback;
DROP TABLE IF EXISTS channels_in_folder;
DROP TABLE IF EXISTS rss_css_channel;
DROP TABLE IF EXISTS rss_folder;
DROP TABLE IF EXISTS user_subscription_channel;
DROP TABLE IF EXISTS morpheme_link_mapping;
DROP TABLE IF EXISTS morpheme;
DROP TABLE IF EXISTS rss_item;
DROP TABLE IF EXISTS rss_channel;
DROP TABLE IF EXISTS news;
DROP TABLE IF EXISTS user;
DROP TABLE IF EXISTS user_omninews_subscription;

CREATE TABLE `user` (
	`user_id` INT NOT NULL AUTO_INCREMENT,
	`user_email` VARCHAR(255) UNIQUE NOT NULL,
	`user_display_name` VARCHAR(100),
	`user_photo_url` TEXT,
	`user_social_login_provider` ENUM('google', 'kakao', 'apple') NOT NULL,
	`user_social_provider_id` VARCHAR(255) NOT NULL,
	`user_access_token` TEXT,
	`user_refresh_token` TEXT,
	`user_access_token_expires_at` DATETIME,
    `user_refresh_token_expires_at` DATETIME,
	`user_status` ENUM('active', 'inactive', 'suspended', 'deleted') DEFAULT 'active',
	`user_role` ENUM('user', 'admin', 'editor') DEFAULT 'user',
	`user_theme` ENUM('light', 'dark', 'blue', 'paper') DEFAULT 'paper',
	`user_notification_push` BOOLEAN NOT NULL DEFAULT FALSE,
    `user_fcm_token` VARCHAR(255) NULL,
	`user_articles_read` INT DEFAULT 0,
	`user_last_active_at` DATETIME,
	`user_created_at` DATETIME,
	`user_updated_at` DATETIME,
    PRIMARY KEY (user_id)
);


CREATE TABLE `omninews_subscription` (
    `omninews_subscription_id`	INT	NOT NULL AUTO_INCREMENT,
	`user_id`	INT	NULL	DEFAULT 0,
    `omninews_subscription_transaction_id`	VARCHAR(255)	NULL,
	`omninews_subscription_status`	BOOLEAN	NULL,
    `omninews_subscription_product_id`	VARCHAR(255)	NULL,
	`omninews_subscription_auto_renew`	BOOLEAN	NULL,
	`omninews_subscription_platform`	VARCHAR(255)	NULL,
  `omninews_subscription_device_id` VARCHAR(255) NULL,
  `omninews_subscription_device_model` VARCHAR(255) NULL,
    `omninews_subscription_start_date`	DATETIME	NULL,
    `omninews_subscription_renew_date`	DATETIME	NULL,
    `omninews_subscription_end_date`	DATETIME	NULL,
	`omninews_subscription_is_sandbox`	BOOLEAN	NULL,
	PRIMARY KEY (omninews_subscription_id),
    FOREIGN KEY (`user_id`) REFERENCES `user`(`user_id`) ON DELETE CASCADE
);

CREATE TABLE `user_subscription_channel` (
	`user_sub_channel_id` INT NOT NULL AUTO_INCREMENT,
	`user_id` INT NULL,
	`channel_id` INT NULL,
    UNIQUE (user_id, channel_id),
    PRIMARY KEY (user_sub_channel_id),
    FOREIGN KEY (`user_id`) REFERENCES `user`(`user_id`) ON DELETE CASCADE,
    FOREIGN KEY (`channel_id`) REFERENCES `rss_channel`(`channel_id`) ON DELETE CASCADE
);

CREATE TABLE `rss_folder` (
	`folder_id` INT NOT NULL AUTO_INCREMENT,
	`folder_name` VARCHAR(50) NULL,
    `user_id` INT NULL,
    PRIMARY KEY (folder_id),
    FOREIGN KEY (`user_id`) REFERENCES `user`(`user_id`) ON DELETE CASCADE
);

CREATE TABLE `channels_in_folder` (
    `channels_in_folder_id` INT NOT NULL AUTO_INCREMENT,
    `folder_id` INT NULL,
    `channel_id` INT NULL,
    PRIMARY KEY (channels_in_folder_id),
    FOREIGN KEY (`folder_id`) REFERENCES `rss_folder`(`folder_id`) ON DELETE CASCADE,
    FOREIGN KEY (`channel_id`) REFERENCES `rss_channel`(`channel_id`) ON DELETE CASCADE
);

CREATE TABLE `news` (
	`news_id` INT NOT NULL AUTO_INCREMENT,
	`news_title` VARCHAR(200) NULL,
	`news_description` VARCHAR(1000) NULL,
    `news_summary` VARCHAR(1000) NULL,
	`news_link` VARCHAR(1000) NULL,
	`news_source` VARCHAR(50) NULL,
	`news_pub_date` DATETIME NULL,
	`news_image_link` VARCHAR(1000) NULL,
	`news_category` VARCHAR(10) NULL,
    PRIMARY KEY (news_id)
);

CREATE TABLE `rss_channel` (
	`channel_id` INT NOT NULL AUTO_INCREMENT,
	`channel_title` VARCHAR(100) NULL,
	`channel_description` VARCHAR(2000) NULL,
	`channel_link` VARCHAR(1000) NULL,
	`channel_image_url` VARCHAR(1000) NULL,
	`channel_language` VARCHAR(10) NULL,
	`rss_generator` VARCHAR(300) NULL,
	`channel_rank` INT NULL,
    `channel_rss_link` VARCHAR(500) UNIQUE,
	PRIMARY KEY (`channel_id`)
);

CREATE TABLE `rss_item` (
	`rss_id` INT NOT NULL AUTO_INCREMENT,
	`channel_id` INT NULL,
	`rss_title` VARCHAR(200) NULL,
	`rss_description` VARCHAR(1000) NULL,
	`rss_link` VARCHAR(1000) NULL,
	`rss_author` VARCHAR(200) NULL COMMENT 'dc:creator, author',
	`rss_pub_date` DATETIME NULL,
	`rss_rank` INT NULL,
	`rss_image_link` VARCHAR(1500) NULL,
	PRIMARY KEY (`rss_id`),
    FOREIGN KEY (`channel_id`) REFERENCES `rss_channel`(`channel_id`) ON DELETE CASCADE
);

CREATE TABLE `embedding` (
    `embedding_id` INT NOT NULL AUTO_INCREMENT,
    `embedding_value` BLOB NOT NULL,
    `channel_id` INT NULL UNIQUE,
    `rss_id` INT NULL UNIQUE,
    `news_id` INT NULL UNIQUE,
    `embedding_source_rank` INT NOT NULL,
    PRIMARY KEY (`embedding_id`),
    FOREIGN KEY (`channel_id`) REFERENCES `rss_channel`(`channel_id`) ON DELETE CASCADE,
    FOREIGN KEY (`rss_id`) REFERENCES `rss_item`(`rss_id`) ON DELETE CASCADE,
    FOREIGN KEY (`news_id`) REFERENCES `news`(`news_id`) ON DELETE CASCADE
);

CREATE TABLE `feedback` (
    `feedback_id` INT NOT NULL AUTO_INCREMENT,
    `feedback_email` VARCHAR(100) NULL,
    `feedback_content` VARCHAR(2000) NOT NULL,
    PRIMARY KEY (`feedback_id`)
);


CREATE TABLE `rss_css_channel` (
	`channel_id` INT NOT NULL,
	`item_title_css` VARCHAR(200) NULL,
	`item_description_css` VARCHAR(200) NULL,
	`item_link_css` VARCHAR(200) NULL,
	`item_author_css` VARCHAR(200) NULL,
	`item_pub_date_css` VARCHAR(200) NULL,
	`item_image_css` VARCHAR(200) NULL,
    PRIMARY KEY (channel_id),
    FOREIGN KEY (`channel_id`) REFERENCES `rss_channel`(`channel_id`) ON DELETE CASCADE
);
