#!/usr/bin/env osascript -l JavaScript
/**
 * create_presentation.js
 * Creates a Keynote presentation about the automating-mac-apps plugin
 * with alternating text and image slides.
 * This is from a working example script that was written by CLAUDE using this skill.
 */

'use strict';

const Keynote = Application("Keynote");
Keynote.includeStandardAdditions = true;

const IMAGES_PATH = "/Users/richardhightower/clients/spillwave/src/skill-foundary-agent/using_apple_automation_foundary/automating-mac-apps-plugin/presentation_images";

function run() {
  try {
    Keynote.activate();
    delay(0.5);

    // Create new document with White theme
    const doc = Keynote.Document({
      documentTheme: Keynote.themes["White"],
      width: 1920,
      height: 1080
    });
    Keynote.documents.push(doc);
    delay(0.5);

    const activeDoc = Keynote.documents[0];

    // Get master slides
    const titleSubtitleMaster = activeDoc.masterSlides["Title & Subtitle"];
    const titleBulletsMaster = activeDoc.masterSlides["Title & Bullets"];
    const blankMaster = activeDoc.masterSlides["Blank"];
    const photoMaster = activeDoc.masterSlides["Photo - Horizontal"];

    // ========== SLIDE 1: Title Slide ==========
    const slide1 = activeDoc.slides[0];
    slide1.defaultTitleItem().objectText = "Automating Mac Apps Plugin";
    slide1.defaultBodyItem().objectText = "Claude Code Plugin for macOS Automation\nJXA • PyXA • AppleScript";
    delay(0.2);

    // ========== SLIDE 2: Cover Image ==========
    const slide2 = Keynote.Slide({ baseSlide: blankMaster });
    activeDoc.slides.push(slide2);
    delay(0.2);

    // Add cover image
    const coverImg = Keynote.Image({
      file: Path(IMAGES_PATH + "/cover.png"),
      position: { x: 160, y: 40 },
      width: 1600
    });
    slide2.images.push(coverImg);
    delay(0.2);

    // ========== SLIDE 3: What is this Plugin? ==========
    const slide3 = Keynote.Slide({ baseSlide: titleBulletsMaster });
    activeDoc.slides.push(slide3);
    slide3.defaultTitleItem().objectText = "What is this Plugin?";
    slide3.defaultBodyItem().objectText = [
      "• Automates 16+ Mac applications from Claude Code",
      "• Uses JavaScript for Automation (JXA) and Python (PyXA)",
      "• Provides ready-to-use scripts and reference docs",
      "• Includes commands, agents, and skills",
      "• Installed from GitHub: SpillwaveSolutions/automating-mac-apps-plugin"
    ].join("\n");
    delay(0.2);

    // ========== SLIDE 4: Architecture Diagram ==========
    const slide4 = Keynote.Slide({ baseSlide: blankMaster });
    activeDoc.slides.push(slide4);
    delay(0.2);

    // Add title text box
    const archTitle = Keynote.TextItem({
      objectText: "Plugin Architecture",
      position: { x: 760, y: 20 },
      width: 400,
      height: 60
    });
    slide4.textItems.push(archTitle);

    // Add architecture image
    const archImg = Keynote.Image({
      file: Path(IMAGES_PATH + "/architecture.png"),
      position: { x: 160, y: 90 },
      width: 1600
    });
    slide4.images.push(archImg);
    delay(0.2);

    // ========== SLIDE 5: Supported Applications ==========
    const slide5 = Keynote.Slide({ baseSlide: titleBulletsMaster });
    activeDoc.slides.push(slide5);
    slide5.defaultTitleItem().objectText = "Supported Applications";
    slide5.defaultBodyItem().objectText = [
      "Productivity: Calendar, Notes, Reminders, Mail",
      "iWork Suite: Numbers, Keynote, Pages",
      "Microsoft Office: Excel, PowerPoint, Word",
      "Communication: Contacts, Messages",
      "Browsers: Chrome, Safari",
      "Media: Voice Memos"
    ].join("\n");
    delay(0.2);

    // ========== SLIDE 6: Skills Overview Diagram ==========
    const slide6 = Keynote.Slide({ baseSlide: blankMaster });
    activeDoc.slides.push(slide6);
    delay(0.2);

    // Add title
    const skillsTitle = Keynote.TextItem({
      objectText: "16 Automation Skills",
      position: { x: 760, y: 20 },
      width: 400,
      height: 60
    });
    slide6.textItems.push(skillsTitle);

    // Add skills overview image
    const skillsImg = Keynote.Image({
      file: Path(IMAGES_PATH + "/skills_overview.png"),
      position: { x: 160, y: 90 },
      width: 1600
    });
    slide6.images.push(skillsImg);
    delay(0.2);

    // ========== SLIDE 7: How It Works ==========
    const slide7 = Keynote.Slide({ baseSlide: titleBulletsMaster });
    activeDoc.slides.push(slide7);
    slide7.defaultTitleItem().objectText = "How It Works";
    slide7.defaultBodyItem().objectText = [
      "1. You ask Claude to automate a Mac app task",
      "2. Claude loads the relevant skill (e.g., automating-notes)",
      "3. The skill provides JXA/PyXA code patterns",
      "4. Claude generates and executes the automation script",
      "5. The Mac app performs the requested action"
    ].join("\n");
    delay(0.2);

    // ========== SLIDE 8: Workflow Diagram ==========
    const slide8 = Keynote.Slide({ baseSlide: blankMaster });
    activeDoc.slides.push(slide8);
    delay(0.2);

    // Add title
    const workflowTitle = Keynote.TextItem({
      objectText: "Automation Workflow",
      position: { x: 760, y: 20 },
      width: 400,
      height: 60
    });
    slide8.textItems.push(workflowTitle);

    // Add workflow image
    const workflowImg = Keynote.Image({
      file: Path(IMAGES_PATH + "/workflow.png"),
      position: { x: 160, y: 90 },
      width: 1600
    });
    slide8.images.push(workflowImg);
    delay(0.2);

    // ========== SLIDE 9: Example Use Cases ==========
    const slide9 = Keynote.Slide({ baseSlide: titleBulletsMaster });
    activeDoc.slides.push(slide9);
    slide9.defaultTitleItem().objectText = "Example Use Cases";
    slide9.defaultBodyItem().objectText = [
      "• Create a note in Notes app about a meeting",
      "• Set a reminder to call someone back",
      "• Generate a Numbers spreadsheet from data",
      "• Build a Keynote presentation (like this one!)",
      "• Send emails with attachments via Mail",
      "• Manage calendar events and invites"
    ].join("\n");
    delay(0.2);

    // ========== SLIDE 10: Getting Started ==========
    const slide10 = Keynote.Slide({ baseSlide: titleBulletsMaster });
    activeDoc.slides.push(slide10);
    slide10.defaultTitleItem().objectText = "Getting Started";
    slide10.defaultBodyItem().objectText = [
      "Install: /plugins install SpillwaveSolutions/automating-mac-apps-plugin",
      "Enable: Check settings.json enabledPlugins",
      "Use: Just ask Claude to automate Mac apps!",
      "",
      "GitHub: github.com/SpillwaveSolutions/automating-mac-apps-plugin"
    ].join("\n");
    delay(0.2);

    return "Created Keynote presentation with 10 slides (alternating text and diagrams)";

  } catch (error) {
    return "Error: " + error.message;
  }
}
