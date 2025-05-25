import tkinter as tk
from tkinter import filedialog
import json
from uuid import uuid4

from utils.option_select import option, UPLINE, CLEAR_LINE


VERSION = 3


def select_links_json_file():
    root = tk.Tk()
    root.withdraw()  # 隐藏主窗口
    root.attributes('-topmost', True)  # 确保对话框在最前面
    
    try:
        file_path = filedialog.askopenfilename(
            title="选择快捷方式配置文件",
            filetypes=[("配置文件", "*.links.json"), ("所有文件", "*.*")],
            parent=root
        )
            
        return file_path
        
    except Exception:
        return None
    finally:
        root.destroy()  # 确保销毁tkinter窗口


def load_conf(path):
    global title, stop_mark
    
    try:
        conf_file = open(path, "r", encoding="utf-8")
        content = conf_file.readlines()
        conf_file.close()

    except FileNotFoundError:
        title = f"{path} 文件不存在或无法读取"
        return

    try:
        conf: dict = json.loads("".join(content))
    except json.JSONDecodeError:
        title = f"{path} 无法作为json文件读取，请检查是否有语法错误"
        return

    new_conf = {}

    new_conf["version"] = VERSION
    if version := conf.get("version"):
        try:
            version_int = int(version)
        except ValueError:
            print(f"版本号 {version} 不是一个有效的整数，将使用版本号 {VERSION}")
        else:
            if version_int < VERSION:
                print(f"版本号 {version_int} 低于当前版本号，将使用版本号 {VERSION}")
            else:
                new_conf["version"] = version_int
        
    else:
        print(f"未找到版本号，将使用版本号 {VERSION}")

    new_conf["tags"] = []
    if tags := conf.get("tags"):
        new_conf["tags"] = tags
    else:
        print("无法获取标签，将初始化标签")
        
    
    new_conf["program_links"] = []
    if program_links := conf.get("program_links"):
        for index, program_link in enumerate(program_links):
            index += 1
            
            new_program_link = {}

            new_program_link["name"] = []
            if (name := program_link.get("name")) is not None:
                if isinstance(name, list):
                    new_program_link["name"] = name
                else:
                    new_program_link["name"] = [str(name)]
            else:
                print(f"无法获取第 {index} 个快捷方式的名称，将使用空名称")
                new_program_link["name"] = []

            new_program_link["icon_path"] = ""
            if (icon_path := program_link.get("icon_path")) is not None:
                new_program_link["icon_path"] = icon_path
            else:
                print(f"无法获取第 {index} 个快捷方式的图标路径，将使用空图标路径")
                new_program_link["icon_path"] = ""

            new_program_link["run_command"] = ""
            if (run_command := program_link.get("run_command")) is not None:
                new_program_link["run_command"] = run_command
            else:
                print(f"无法获取第 {index} 个快捷方式的运行命令，将使用空运行命令")
                new_program_link["run_command"] = ""
                
            new_program_link["tags"] = []
            if (tags := program_link.get("tags")) is not None:
                if isinstance(tags, list):
                    for tag in tags:
                        if tag in new_conf["tags"]:
                            new_program_link["tags"].append(tag)
                        
                else:
                    print(f"无法获取第 {index} 个快捷方式的标签，将使用空标签")
                    new_program_link["tags"] = []
            else:
                print(f"无法获取第 {index} 个快捷方式的标签，将使用空标签")
                new_program_link["tags"] = []
                
            new_program_link["uuid"] = ""
            if uuid := program_link.get("uuid"):
                new_program_link["uuid"] = uuid
            else:
                print(f"无法获取第 {index} 个快捷方式的uuid，将使用随机uuid")
                new_program_link["uuid"] = str(uuid4())
                
            new_conf["program_links"].append(new_program_link)

    else:
        print("无法获取快捷方式，将初始化快捷方式")
        new_conf["program_links"] = []

    if new_conf == conf:
        title = "此配置文件无需更新"
        return

    if option(
        {'Y': ['y', 'yes'], 'n': ['n', 'no']},
        "是否保存更新后的配置文件",
        function_for_input=lambda x: x.lower(),
        general_format="{title}[{options}]: ",
        option_format="{option}",
        options_connect="/",
        invalid_input_return='Y',
        keep_options_after_select=True,
    ) == "Y":
        with open(path, "w", encoding="utf-8") as f:
            json.dump(new_conf, f, ensure_ascii=False, indent=4)

        print("配置文件已更新")
        stop_mark = True

title = "BaroBoard 快捷方式配置文件更新"
stop_mark = False
while not stop_mark:
    match option(
        ["选择配置文件", "手动输入配置文件路径", "退出"], 
        title, 
        return_key=True
    ):
        case 0:
            path = select_links_json_file()

        case 1:
            path = input("输入配置文件路径: ")
            print(f"{UPLINE}{CLEAR_LINE}", end="")

        case 2:
            exit()

    if not path.endswith(".links.json"):
        title = f"{path} 不是一个有效的快捷方式配置文件 (.links.json)"
        continue

    load_conf(path)
