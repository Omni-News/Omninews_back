drop table if exists user;
drop table if exists rss_item;
drop table if exists rss_channel;
drop table if exists news;
drop table if exists embedding;
drop table if exists feedback;
drop table if exists morpheme_link_mapping;
drop table if exists morpheme;
drop table if exists user_subscription_channel;
drop table if exists rss_folder;
drop table if exists channels_in_folder;
drop table if exists rss_css_channel;

CREATE TABLE `user` (
	`user_id` INT NOT NULL AUTO_INCREMENT  ,
	`user_email`	VARCHAR(255)	UNIQUE NOT NULL,
	`user_display_name`	VARCHAR(100),
	`user_photo_url`	TEXT,
	`user_social_login_provider`	ENUM('google', 'kakao', 'apple') NOT NULL,
	`user_social_provider_id`	VARCHAR(255) NOT NULL, -- Apple은 로그인 시 이 속성으로 비교해야 함.
	`user_access_token`	TEXT,
	`user_refresh_token`	TEXT,
	`user_access_token_expires_at`	DATETIME,
  `user_refresh_token_expires_at`	DATETIME,
	`user_status`	ENUM('active', 'inactive', 'suspended, deleted')	DEFAULT 'active',
	`user_role`	ENUM('user', 'admin', 'editor')	DEFAULT 'user',
	`user_theme`	ENUM('light', 'dark', 'blue', 'paper')	DEFAULT 'paper',
	`user_notification_push`	BOOLEAN NOT NULL	DEFAULT FALSE,
  `user_fcm_token` VARCHAR(255) NULL,
	`user_articles_read`	INT	DEFAULT 0,
	`user_last_active_at`	DATETIME    ,
  `user_subscription_product_id`  VARCHAR(100) NULL,
  `user_subscription_receipt_data`  VARCHAR(3000) NULL,
  `user_subscription_platform`  ENUM('ios', 'android') NULL,
  `user_subscription_is_test` BOOLEAN NULL,
	`user_subscription_plan`    BOOLEAN	DEFAULT FALSE,
	`user_subscription_start_date`	DATETIME	NULL,
	`user_subscription_end_date`	DATETIME	NULL,
	`user_subscription_auto_renew`	BOOLEAN	DEFAULT FALSE,
	`user_created_at`	DATETIME	,
	`user_updated_at`	DATETIME,
    PRIMARY KEY (user_id)
);


CREATE TABLE `news` (
	`news_id`	INT	NOT NULL AUTO_INCREMENT,
	`news_title`	VARCHAR(200)	NULL,
	`news_description`	VARCHAR(1000)	NULL,
  `news_summary` VARCHAR(1000) NULL,
	`news_link`	VARCHAR(1000)	NULL,
	`news_source`	VARCHAR(50)	NULL,
	`news_pub_date`	DATETIME	NULL,
	`news_image_link`	VARCHAR(1000)	NULL,
	`news_category`	VARCHAR(10)	NULL,
    PRIMARY KEY (news_id)
);

CREATE TABLE `rss_item` (
	`rss_id`	INT NOT NULL AUTO_INCREMENT,
	`channel_id`	INT	NULL	DEFAULT 0,
	`rss_title`	VARCHAR(200)	NULL,
	`rss_description`	VARCHAR(1000)	NULL,
	`rss_link`	VARCHAR(1000)	NULL,
	`rss_author`	VARCHAR(200)	NULL	COMMENT 'dc:creator, author',
	`rss_pub_date`	DATETIME	NULL,
	`rss_rank`	INT	NULL,
	`rss_image_link`	VARCHAR(1500)	NULL,
	PRIMARY KEY (`rss_id`)
);

CREATE TABLE `rss_channel` (
	`channel_id`	INT NOT NULL AUTO_INCREMENT,
	`channel_title`	VARCHAR(100)	NULL ,
	`channel_description`	VARCHAR(2000)	NULL,
	`channel_link`	VARCHAR(1000)	NULL,
	`channel_image_url`	VARCHAR(1000)	NULL,
	`channel_language`	VARCHAR(10)	NULL,
	`rss_generator`	VARCHAR(300)	NULL,
	`channel_rank`	INT	NULL,
  `channel_rss_link` VARCHAR(500) UNIQUE ,
	PRIMARY KEY (`channel_id`)
);

CREATE TABLE `embedding` (
    `embedding_id` INT NOT NULL AUTO_INCREMENT,
    `embedding_value` BLOB NOT NULL,
    `channel_id` INT NULL UNIQUE,
    `rss_id` INT NULL UNIQUE,
    `news_id` INT NULL UNIQUE,
    `embedding_source_rank` INT NOT NULL,
    PRIMARY KEY (`embedding_id`)
);

CREATE TABLE `feedback` (
    `feedback_id`   INT NOT NULL AUTO_INCREMENT,
    `feedback_email`    VARCHAR(100)    NULL,
    `feedback_content`  VARCHAR(2000) NOT NULL,
    PRIMARY KEY (`feedback_id`)
);


CREATE TABLE `user_subscription_channel` (
	`user_sub_channel_id` INT NOT NULL AUTO_INCREMENT,
	`user_id`	INT	NULL	DEFAULT 0,
	`channel_id`	INT	NULL	DEFAULT 0,
    UNIQUE (user_id, channel_id),
    PRIMARY KEY (user_sub_channel_id)
);

CREATE TABLE `rss_folder` (
	`folder_id`	INT	NOT NULL AUTO_INCREMENT,
	`folder_name`	VARCHAR(50)	NULL,
  `user_id` INT NULL DEFAULT 0,
  PRIMARY KEY (folder_id)
);

CREATE TABLE `channels_in_folder` (
  `channels_in_folder_id` INT NOT NULL AUTO_INCREMENT,
  `folder_id` INT NULL DEFAULT 0,
  `channel_id` INT NULL DEFAULT 0,
  PRIMARY KEY (channels_in_folder_id)
);

CREATE TABLE `rss_css_channel` (
	`channel_id`	INT	DEFAULT 0,
	`item_title_css`	VARCHAR(200)	NULL,
	`item_description_css`	VARCHAR(200)	NULL,
	`item_link_css`	VARCHAR(200)	NULL,
	`item_author_css`	VARCHAR(200)	NULL,
	`item_pub_date_css`	VARCHAR(200)	NULL,
	`item_image_css`	VARCHAR(200)	NULL,
  PRIMARY KEY (channel_id)
);
