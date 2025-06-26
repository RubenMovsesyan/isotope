import numpy as np
import math
from scipy.spatial.transform import Rotation as R
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D

def from_axis_angle(axis, angle):
    (s, c) = (np.sin(angle / 2), np.cos(angle / 2))
    scalar = c
    vector = axis * s
    return [vector[0], vector[1], vector[2], scalar]

def mul_quat_vec(q, v):
    vec = np.array([q[0], q[1], q[2]])
    tmp = np.cross(vec, v) + (v * q[3])
    return (np.cross(vec, tmp) * 2) + v


def find_planes(
    eye,
    target,
    up,
    aspect,
    fovy,
    znear,
    zfar,
):
    position = eye
    front = target / np.linalg.norm(target)
    up_norm = up / np.linalg.norm(up)
    right = np.cross(front, up_norm)
    right /= np.linalg.norm(right)

    print("Front:", front)
    print("Up:", up_norm)
    print("Right:", right)

    half_fov_rad = np.deg2rad(fovy) * 0.5
    half_fov_h_rad = np.atan(np.tan(half_fov_rad) * aspect)

    print("Half FOV rad", half_fov_rad)
    print("Half FOV H rad", half_fov_h_rad)


    fig = plt.figure()
    ax = fig.add_subplot(111, projection='3d')
    ax.set_box_aspect([1, 1, 1])

    # Add camera position and direction for reference
    ax.scatter(*position, color='green', s=100, label='Camera')
    ax.quiver(*position, *front, length=10, color='black', label='Front')
    ax.quiver(*position, *up_norm, length=10, color='red', label='Up')
    ax.quiver(*position, *right, length=10, color='orange', label='Right')

    # top_edge_rotation = from_axis_angle(right, half_fov_rad)
    # top_edge_direction = mul_quat_vec(top_edge_rotation, front)

    # top_normal = np.cross(top_edge_direction, right)
    # top_normal /= np.linalg.norm(top_normal)
    # top_rotation = from_axis_angle(right, -half_fov_rad)
    # top_normal = mul_quat_vec(top_rotation, -up_norm)
    # top_normal /= np.linalg.norm(top_normal)

    first_rotation = from_axis_angle(right, half_fov_rad)
    top_direction = mul_quat_vec(first_rotation, front)
    second_rotation = from_axis_angle(right, -math.pi / 2)
    top_normal = mul_quat_vec(second_rotation, top_direction)
    top_normal /= np.linalg.norm(top_normal)

    ax.quiver(*position, *top_normal, length=10, color='purple', label='Top Normal')

    # bottom_edge_rotation = from_axis_angle(right, -half_fov_rad)
    # bottom_edge_direction = mul_quat_vec(bottom_edge_rotation, front)

    # bottom_normal = np.cross(right, bottom_edge_direction)
    # bottom_normal /= np.linalg.norm(bottom_normal)

    first_rotation = from_axis_angle(right, -half_fov_rad)
    bottom_direction = mul_quat_vec(first_rotation, front)
    second_rotation = from_axis_angle(right, math.pi / 2)
    bottom_normal = mul_quat_vec(second_rotation, bottom_direction)
    bottom_normal /= np.linalg.norm(bottom_normal)


    left_edge_rotation = from_axis_angle(up_norm, half_fov_h_rad)
    left_edge_direction = mul_quat_vec(left_edge_rotation, front)

    left_normal = np.cross(left_edge_direction, up_norm)
    left_normal /= np.linalg.norm(left_normal)


    right_edge_rotation = from_axis_angle(up_norm, -half_fov_h_rad)
    right_edge_direction = mul_quat_vec(right_edge_rotation, front)

    right_normal = np.cross(right_edge_direction, up_norm)
    right_normal /= np.linalg.norm(right_normal)

    print("Top Normal:", top_normal)
    print("Bottom Normal:", bottom_normal)
    print("Left Normal:", left_normal)
    print("Right Normal:", right_normal)



    x = np.linspace(-20, 20, 50)
    y = np.linspace(-20, 20, 50)
    x, y = np.meshgrid(x, y)

    a, b, c = top_normal
    d = np.dot(top_normal, position)

    if abs(c) > 1e-6:
        z = (d - a * x - b * y) / c
        ax.plot_wireframe(x, y, z, alpha=0.5)


    a, b, c = bottom_normal
    d = np.dot(bottom_normal, position)

    if abs(c) > 1e-6:
        z = (d - a * x - b * y) / c
        ax.plot_wireframe(x, y, z, alpha=0.5)


    # a, b, c = left_normal
    # d = np.dot(left_normal, position)

    # if abs(c) > 1e-6:
    #     z = (d - a * x - b * y) / c
    #     ax.plot_wireframe(x, y, z, alpha=0.5)


    # a, b, c = right_normal
    # d = np.dot(right_normal, position)

    # if abs(c) > 1e-6:
    #     z = (d - a * x - b * y) / c
    #     ax.plot_wireframe(x, y, z, alpha=0.5)


    ax.set_xlabel('X')
    ax.set_ylabel('Y')
    ax.set_zlabel('Z')
    ax.legend()

    plt.show()


if __name__ == '__main__':
    find_planes(
        np.array([10, 10, 10]),
        np.array([-0.57735026, -0.57735026, -0.57735026]),
        np.array([0, 1, 0]),
        1.8695229,
        90,
        0.1,
        100
    )
