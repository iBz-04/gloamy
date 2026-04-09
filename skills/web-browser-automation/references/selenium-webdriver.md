# Selenium WebDriver Automation

## Installation
```bash
pip install selenium
# ChromeDriver automatically managed in v4.11+
```

## Basic Usage
```python
from selenium import webdriver
from selenium.webdriver.common.by import By

driver = webdriver.Chrome()
driver.get("https://example.com")
title = driver.title
print(f"Page title: {title}")
driver.quit()
```

## Browser Configuration
```python
from selenium.webdriver.chrome.options import Options

options = Options()
options.add_argument("--headless=new")
options.add_argument("--no-sandbox")
options.add_argument("--disable-dev-shm-usage")
options.add_argument("--window-size=1920,1080")

driver = webdriver.Chrome(options=options)
```

## Element Location and Interaction
```python
# Find elements
element = driver.find_element(By.ID, "username")
elements = driver.find_elements(By.CLASS_NAME, "item")

# Interactions
element.click()
element.send_keys("text input")
element.clear()

# Advanced selectors
from selenium.webdriver.common.by import By
email_field = driver.find_element(By.CSS_SELECTOR, "input[type='email']")
submit_btn = driver.find_element(By.XPATH, "//button[@type='submit']")
```

## Waiting Strategies
```python
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

# Explicit wait
wait = WebDriverWait(driver, 10)
element = wait.until(EC.presence_of_element_located((By.ID, "results")))

# Fluent wait with conditions
element = WebDriverWait(driver, 10, poll_frequency=1).until(
    lambda driver: driver.find_element(By.ID, "dynamic-element")
)
```

## Actions and Advanced Interactions
```python
from selenium.webdriver.common.action_chains import ActionChains

actions = ActionChains(driver)
actions.move_to_element(menu).click(submenu).perform()

# Drag and drop
actions.drag_and_drop(source, target).perform()

# Keyboard shortcuts
from selenium.webdriver.common.keys import Keys
element.send_keys(Keys.CONTROL, 'a')  # Select all
element.send_keys(Keys.DELETE)  # Delete
```

## Window and Tab Management
```python
# Handle multiple windows
main_window = driver.current_window_handle
driver.find_element(By.LINK_TEXT, "Open New Window").click()

for handle in driver.window_handles:
    if handle != main_window:
        driver.switch_to.window(handle)
        print(f"New window title: {driver.title}")
        driver.close()

driver.switch_to.window(main_window)
```

## File Upload and Download
```python
# File upload
file_input = driver.find_element(By.CSS_SELECTOR, "input[type='file']")
file_input.send_keys("/path/to/file.pdf")

# Download handling
from selenium.webdriver.chrome.options import Options

options = Options()
options.add_experimental_option("prefs", {
    "download.default_directory": "/tmp/downloads",
    "download.prompt_for_download": False,
    "download.directory_upgrade": True
})

driver = webdriver.Chrome(options=options)
```

## Cookie Management
```python
# Get all cookies
cookies = driver.get_cookies()

# Add cookie
driver.add_cookie({"name": "session", "value": "abc123"})

# Delete cookie
driver.delete_cookie("session")
driver.delete_all_cookies()
```

## Screenshot and Visual Testing
```python
# Full page screenshot
driver.save_screenshot("page.png")

# Element screenshot
element = driver.find_element(By.ID, "content")
element.screenshot("element.png")

# PDF generation (Chrome only)
from selenium.webdriver.chrome.options import Options

options = Options()
options.add_experimental_option("prefs", {
    "printing.print_preview_sticky_settings.appState": json.dumps({
        "recentDestinations": [{"id": "Save as PDF", "origin": "local"}],
        "selectedDestinationId": "Save as PDF",
        "version": 2
    })
})
```

## Parallel Testing
```python
import threading
from selenium import webdriver

def test_browser(url):
    driver = webdriver.Chrome()
    driver.get(url)
    title = driver.title
    driver.quit()
    return title

# Run tests in parallel
urls = ["https://site1.com", "https://site2.com", "https://site3.com"]
threads = []

for url in urls:
    thread = threading.Thread(target=test_browser, args=(url,))
    threads.append(thread)
    thread.start()

for thread in threads:
    thread.join()
```

## Grid and Remote Execution
```python
from selenium.webdriver.remote.webdriver import WebDriver as RemoteWebDriver
from selenium.webdriver.common.desired_capabilities import DesiredCapabilities

# Connect to Selenium Grid
driver = RemoteWebDriver(
    command_executor="http://localhost:4444/wd/hub",
    desired_capabilities=DesiredCapabilities.CHROME
)

driver.get("https://example.com")
```

## Page Object Model
```python
class LoginPage:
    def __init__(self, driver):
        self.driver = driver
        self.username_field = (By.ID, "username")
        self.password_field = (By.ID, "password")
        self.login_button = (By.ID, "login")

    def login(self, username, password):
        self.driver.find_element(*self.username_field).send_keys(username)
        self.driver.find_element(*self.password_field).send_keys(password)
        self.driver.find_element(*self.login_button).click()

# Usage
driver = webdriver.Chrome()
login_page = LoginPage(driver)
login_page.login("user@example.com", "password")
```